//! `openAgentNativeConfiguration` (specs/002 T10).
//!
//! The caller names only a `targetId`. The target must be declared in the profile's
//! fresh module descriptor; no path, command or argument comes from the request.
//! Opening goes through an opener boundary. V1 ships no qualified opener, so a
//! declared target is refused with an honest unavailable error, and an undeclared
//! target is rejected before the opener is consulted.
#![allow(clippy::result_large_err)]

use super::agent_control_operator::{
    AgentControlApiState, fresh_descriptor, no_store_middleware, validate_profile_id,
};
use super::{ApiState, operator_error};
use axum::{
    Json, Router,
    extract::{FromRef, Path, State},
    http::{HeaderMap, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::post,
};
use serde_json::json;
use std::sync::Arc;

/// Opens one module-declared native configuration target in the OS shell.
pub(crate) trait NativeConfigOpener: Send + Sync {
    fn open(&self, module_id: &str, target_id: &str) -> Result<(), ()>;
}

pub(crate) struct UnqualifiedOpener;

impl NativeConfigOpener for UnqualifiedOpener {
    fn open(&self, _module_id: &str, _target_id: &str) -> Result<(), ()> {
        Err(())
    }
}

#[derive(Clone)]
struct NativeConfigApiState {
    control: AgentControlApiState,
    opener: Arc<dyn NativeConfigOpener>,
}

impl FromRef<ApiState> for NativeConfigApiState {
    fn from_ref(state: &ApiState) -> Self {
        Self {
            control: AgentControlApiState::from_ref(state),
            opener: Arc::new(UnqualifiedOpener),
        }
    }
}

pub(super) fn routes() -> Router<ApiState> {
    routes_with_state::<ApiState>()
        .route_layer(middleware::from_fn(super::authenticate))
        .route_layer(middleware::from_fn(no_store_middleware))
}

fn routes_with_state<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    NativeConfigApiState: FromRef<S>,
{
    Router::new().route(
        "/v1/agent-profiles/{agent_profile_id}/native-config/{target_id}/open",
        post(open_native_configuration),
    )
}

#[cfg(test)]
pub(super) fn test_router(control: AgentControlApiState) -> Router {
    routes_with_state::<NativeConfigApiState>()
        .route_layer(middleware::from_fn(super::authenticate))
        .route_layer(middleware::from_fn(no_store_middleware))
        .with_state(NativeConfigApiState {
            control,
            opener: Arc::new(UnqualifiedOpener),
        })
}

async fn open_native_configuration(
    State(state): State<NativeConfigApiState>,
    headers: HeaderMap,
    Path((agent_profile_id, target_id)): Path<(String, String)>,
) -> Response {
    if let Err(response) = validate_profile_id(&agent_profile_id) {
        return response;
    }
    if target_id.trim().is_empty()
        || target_id.len() > 128
        || target_id.chars().any(char::is_control)
        || target_id.contains(['/', '\\'])
    {
        return operator_error(
            StatusCode::BAD_REQUEST,
            "INVALID_ARGUMENT",
            "Native configuration target ID is invalid",
        );
    }
    let descriptor = match fresh_descriptor(&state.control, &headers, &agent_profile_id).await {
        Ok(descriptor) => descriptor,
        Err(response) => return response,
    };
    let declared = descriptor
        .configuration
        .native_config_targets
        .iter()
        .any(|target| target.target_id == target_id);
    if !declared {
        return operator_error(
            StatusCode::BAD_REQUEST,
            "INVALID_ARGUMENT",
            "Native configuration target is not declared by this agent module",
        );
    }
    let opener = state.opener.clone();
    let module_id = descriptor.module_id.clone();
    let target = target_id.clone();
    let opened = tokio::task::spawn_blocking(move || opener.open(&module_id, &target)).await;
    match opened {
        Ok(Ok(())) => Json(json!({
            "agent_profile_id": agent_profile_id,
            "target_id": target_id,
            "opened": true,
        }))
        .into_response(),
        _ => operator_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "DEPENDENCY_UNAVAILABLE",
            "No qualified native configuration opener exists for this agent module",
        ),
    }
}
