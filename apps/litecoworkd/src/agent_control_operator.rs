//! AgentControlDescriptor routes.
//!
//! A descriptor is served only for a profile whose stored provider identity selects
//! a trusted AgentModule (today: OpenCode 2.0.32) and only while its stored revision
//! is unexpired. The routes never manufacture a descriptor from a catalog or
//! RuntimeOffer, so the composer and session-admission boundary stay fail-closed for
//! every module-less agent. Refresh runs one bounded probe under the module budget.
#![allow(clippy::result_large_err)]

use super::{
    ApiState, ensure_workspace_owner_for, map_agent_store_error, operator_error, selected_workspace,
};
use crate::agents::descriptor_service::{AgentDescriptorService, RefreshError};
use crate::agents::module_registry::AgentModuleManifest;
use axum::response::IntoResponse;
use axum::{
    Router,
    body::Body,
    extract::{FromRef, Path, State},
    http::{HeaderMap, Request, StatusCode, header},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
};
use std::sync::Arc;
use storage_core::AgentCatalogStore;

#[derive(Clone)]
pub(super) struct AgentControlApiState {
    store: super::SqliteWorkspaceStore,
    principal_id: String,
    runtime_id: String,
    local_incarnation_id: String,
    service: Arc<AgentDescriptorService>,
}

impl FromRef<ApiState> for AgentControlApiState {
    fn from_ref(state: &ApiState) -> Self {
        Self {
            store: state.store.clone(),
            principal_id: state.principal_id.clone(),
            runtime_id: state.runtime_id.clone(),
            local_incarnation_id: state.local_incarnation_id.clone(),
            service: AgentDescriptorService::shared(),
        }
    }
}

impl AgentControlApiState {
    #[cfg(test)]
    pub(super) fn for_test(
        store: super::SqliteWorkspaceStore,
        principal_id: String,
        service: Arc<AgentDescriptorService>,
    ) -> Self {
        Self {
            store,
            principal_id,
            runtime_id: "agent-control-runtime".to_owned(),
            local_incarnation_id: "agent-control-incarnation".to_owned(),
            service,
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
    AgentControlApiState: FromRef<S>,
{
    Router::new()
        .route(
            "/v1/agent-profiles/{agent_profile_id}/control-descriptor",
            get(get_control_descriptor),
        )
        .route(
            "/v1/agent-profiles/{agent_profile_id}/refresh",
            post(refresh_control_descriptor),
        )
}

#[cfg(test)]
fn test_router(store: super::SqliteWorkspaceStore, principal_id: String) -> Router {
    test_router_with(store, principal_id, AgentDescriptorService::shared())
}

#[cfg(test)]
fn test_router_with(
    store: super::SqliteWorkspaceStore,
    principal_id: String,
    service: Arc<AgentDescriptorService>,
) -> Router {
    routes_with_state::<AgentControlApiState>()
        .route_layer(middleware::from_fn(super::authenticate))
        .route_layer(middleware::from_fn(no_store_middleware))
        .with_state(AgentControlApiState {
            store,
            principal_id,
            runtime_id: "agent-control-runtime".to_owned(),
            local_incarnation_id: "agent-control-incarnation".to_owned(),
            service,
        })
}

async fn get_control_descriptor(
    State(state): State<AgentControlApiState>,
    headers: HeaderMap,
    Path(agent_profile_id): Path<String>,
) -> Response {
    let (module, now) = match resolve_module(&state, &headers, &agent_profile_id).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    let Some(module) = module else {
        // A discovered profile and its offer are observations, not a module-owned
        // AgentControlDescriptor. Without a trusted module nothing is manufactured.
        return unavailable_descriptor();
    };
    let service = state.service.clone();
    let store = state.store.clone();
    let runtime_id = state.runtime_id.clone();
    let incarnation_id = state.local_incarnation_id.clone();
    let current = tokio::task::spawn_blocking(move || {
        service.current(&store, &module, &runtime_id, &incarnation_id, &now)
    })
    .await;
    match current {
        Ok(Ok(Some(descriptor))) => axum::Json(descriptor).into_response(),
        // No unexpired descriptor: the caller must refresh. Never serve a stale one.
        Ok(Ok(None)) => unavailable_descriptor(),
        Ok(Err(error)) => map_agent_store_error(error, "Agent control descriptor is unavailable"),
        Err(_) => unavailable(),
    }
}

async fn refresh_control_descriptor(
    State(state): State<AgentControlApiState>,
    headers: HeaderMap,
    Path(agent_profile_id): Path<String>,
) -> Response {
    let (module, _) = match resolve_module(&state, &headers, &agent_profile_id).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    let Some(module) = module else {
        return unavailable_descriptor();
    };
    let service = state.service.clone();
    let store = state.store.clone();
    let runtime_id = state.runtime_id.clone();
    let incarnation_id = state.local_incarnation_id.clone();
    // The bounded probe runs on a blocking thread, outside any storage transaction.
    let refreshed = tokio::task::spawn_blocking(move || {
        service.refresh(
            &store,
            &module,
            &runtime_id,
            &incarnation_id,
            time::OffsetDateTime::now_utc(),
        )
    })
    .await;
    match refreshed {
        Ok(Ok(descriptor)) => axum::Json(descriptor).into_response(),
        Ok(Err(RefreshError::Throttled)) => operator_error(
            StatusCode::TOO_MANY_REQUESTS,
            "RATE_LIMITED",
            "Agent descriptor refresh is limited to three probes per minute for this module",
        ),
        Ok(Err(RefreshError::Unavailable)) => unavailable_descriptor(),
        Ok(Err(RefreshError::Store)) => operator_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL",
            "Agent control descriptor could not be recorded",
        ),
        Err(_) => unavailable(),
    }
}

/// Validates the path, authorizes the Workspace owner, and resolves the profile's
/// trusted module from the stored provider identity (None when it has no module).
pub(super) async fn resolve_module(
    state: &AgentControlApiState,
    headers: &HeaderMap,
    agent_profile_id: &str,
) -> Result<(Option<AgentModuleManifest>, String), Response> {
    validate_profile_id(agent_profile_id)?;
    let workspace_id = selected_workspace(headers)?;
    let now = super::operator_now()?;
    let store = state.store.clone();
    let principal = state.principal_id.clone();
    let service = state.service.clone();
    let profile_id = agent_profile_id.to_owned();
    let now_for_lookup = now.clone();
    let looked_up = tokio::task::spawn_blocking(move || {
        ensure_workspace_owner_for(&store, &principal, &workspace_id)?;
        let profiles = store
            .list_agent_profiles(&principal, &workspace_id, &now_for_lookup)
            .map_err(|error| map_agent_store_error(error, "Agent profiles are unavailable"))?;
        Ok::<_, Response>(
            profiles
                .into_iter()
                .find(|view| view.profile.agent_profile_id == profile_id)
                .map(|view| {
                    service
                        .module_for_provider(&view.profile.provider_key)
                        .cloned()
                }),
        )
    })
    .await;
    match looked_up {
        Ok(Ok(Some(module))) => Ok((module, now)),
        Ok(Ok(None)) => Err(unavailable_profile()),
        Ok(Err(response)) => Err(response),
        Err(_) => Err(unavailable()),
    }
}

/// The profile's current unexpired module descriptor, or the exact error response.
/// Shared by the sign-in and native-configuration routes so neither can act on a
/// stale or manufactured descriptor.
pub(super) async fn fresh_descriptor(
    state: &AgentControlApiState,
    headers: &HeaderMap,
    agent_profile_id: &str,
) -> Result<crate::agents::control_descriptor::AgentControlDescriptor, Response> {
    let (module, now) = resolve_module(state, headers, agent_profile_id).await?;
    let Some(module) = module else {
        return Err(unavailable_descriptor());
    };
    let service = state.service.clone();
    let store = state.store.clone();
    let runtime_id = state.runtime_id.clone();
    let incarnation_id = state.local_incarnation_id.clone();
    let current = tokio::task::spawn_blocking(move || {
        service.current(&store, &module, &runtime_id, &incarnation_id, &now)
    })
    .await;
    match current {
        Ok(Ok(Some(descriptor))) => Ok(descriptor),
        Ok(Ok(None)) => Err(unavailable_descriptor()),
        Ok(Err(error)) => Err(map_agent_store_error(
            error,
            "Agent control descriptor is unavailable",
        )),
        Err(_) => Err(unavailable()),
    }
}

pub(super) fn validate_profile_id(value: &str) -> Result<(), Response> {
    if value.trim().is_empty()
        || value.len() > 256
        || value.chars().any(char::is_control)
        || value.contains('/')
    {
        return Err(operator_error(
            StatusCode::BAD_REQUEST,
            "INVALID_ARGUMENT",
            "Agent profile ID is invalid",
        ));
    }
    Ok(())
}

pub(super) async fn no_store_middleware(request: Request<Body>, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    response
}

fn unavailable_profile() -> Response {
    operator_error(
        StatusCode::NOT_FOUND,
        "NOT_FOUND",
        "Agent profile is unavailable in this Workspace",
    )
}

fn unavailable_descriptor() -> Response {
    operator_error(
        StatusCode::SERVICE_UNAVAILABLE,
        "DEPENDENCY_UNAVAILABLE",
        "A qualified AgentModule control descriptor is not available for this agent",
    )
}

fn unavailable() -> Response {
    operator_error(
        StatusCode::SERVICE_UNAVAILABLE,
        "DEPENDENCY_UNAVAILABLE",
        "Agent control descriptor is unavailable",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::to_bytes, http::Request};
    use domain_workspace::{CreateWorkspace, EventContext, WorkspaceService};
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::{fs, sync::Arc, time::Duration};
    use storage_core::{AgentEndpointRecord, AgentProfileRecord, BlobPurpose, StoreError};
    use storage_sqlite::{FileBlobStore, SqliteConfig, WorkspaceBlobKey, WorkspaceBlobKeyProvider};
    use tower::ServiceExt;
    use zeroize::Zeroizing;

    const OWNER: &str = "agent-control-owner";
    const WORKSPACE: &str = "agent-control-workspace";
    const RUNTIME: &str = "agent-control-runtime";
    const NOW: &str = "2026-10-18T12:00:00Z";

    struct TestKeys;

    impl WorkspaceBlobKeyProvider for TestKeys {
        fn current_key(
            &self,
            _workspace_id: &str,
            _purpose: BlobPurpose,
        ) -> Result<WorkspaceBlobKey, StoreError> {
            Ok(WorkspaceBlobKey {
                version: 1,
                bytes: Zeroizing::new([0x31; 32]),
            })
        }

        fn key_by_version(
            &self,
            workspace_id: &str,
            purpose: BlobPurpose,
            version: u32,
        ) -> Result<WorkspaceBlobKey, StoreError> {
            if version != 1 {
                return Err(StoreError::Blob("unknown test key version".to_owned()));
            }
            self.current_key(workspace_id, purpose)
        }
    }

    fn fixture() -> (tempfile::TempDir, super::super::SqliteWorkspaceStore) {
        let directory = tempfile::tempdir().expect("temporary descriptor directory");
        #[cfg(unix)]
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700))
            .expect("restrict test directory");
        let store = super::super::SqliteWorkspaceStore::open(
            directory.path().join("state.sqlite3"),
            Arc::new(FileBlobStore::new(directory.path().join("blobs"), TestKeys)),
            SqliteConfig {
                writer_queue_capacity: 8,
                busy_timeout: Duration::from_secs(2),
            },
        )
        .expect("temporary SQLite store opens");
        WorkspaceService::new(store.clone())
            .create(CreateWorkspace {
                workspace_id: WORKSPACE.to_owned(),
                name: "Agent descriptor tests".to_owned(),
                owner_principal_id: OWNER.to_owned(),
                event: EventContext {
                    event_id: "agent-control-workspace-created".to_owned(),
                    origin_runtime_id: RUNTIME.to_owned(),
                    hlc_timestamp: NOW.to_owned(),
                    correlation_id: "agent-control-workspace-created".to_owned(),
                    causation_id: None,
                    recorded_at: NOW.to_owned(),
                },
            })
            .expect("Workspace commits");
        (directory, store)
    }

    async fn request(router: &Router, path: &str) -> Response {
        let mut request = Request::get(path)
            .header("x-workspace-id", WORKSPACE)
            .body(Body::empty())
            .expect("request is valid");
        request
            .extensions_mut()
            .insert(super::super::AuthenticatedLocalPeer);
        router
            .clone()
            .oneshot(request)
            .await
            .expect("route responds")
    }

    fn seed_profile(store: &super::super::SqliteWorkspaceStore) {
        store
            .put_agent_profile(
                AgentProfileRecord {
                    agent_profile_id: "profile-control".to_owned(),
                    provider_key: "CODEX".to_owned(),
                    display_name: "Codex".to_owned(),
                    discovered_at: NOW.to_owned(),
                },
                vec![AgentEndpointRecord {
                    endpoint_id: "endpoint-control".to_owned(),
                    agent_profile_id: "profile-control".to_owned(),
                    protocol: "CLI".to_owned(),
                    topology: "PROCESS_ADAPTER".to_owned(),
                    protocol_version: None,
                    capabilities: serde_json::json!({"input.text":true}),
                }],
            )
            .expect("profile observation persists");
    }

    #[tokio::test]
    async fn profile_catalog_presence_never_manufactures_a_control_descriptor() {
        let (_directory, store) = fixture();
        seed_profile(&store);
        let router = test_router(store, OWNER.to_owned());
        let response = request(
            &router,
            "/v1/agent-profiles/profile-control/control-descriptor",
        )
        .await;
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        let body = to_bytes(response.into_body(), 8192)
            .await
            .expect("bounded error body");
        let value: serde_json::Value = serde_json::from_slice(&body).expect("JSON error");
        assert_eq!(value["code"], "DEPENDENCY_UNAVAILABLE");
        assert!(
            !body.windows(8).any(|window| window == b"endpoint"),
            "private endpoint details must not be returned"
        );
    }

    #[tokio::test]
    async fn refresh_without_a_module_is_read_only_and_no_store() {
        let (_directory, store) = fixture();
        seed_profile(&store);
        let router = test_router(store.clone(), OWNER.to_owned());
        let mut request = Request::post("/v1/agent-profiles/profile-control/refresh")
            .header("x-workspace-id", WORKSPACE)
            .body(Body::empty())
            .expect("request is valid");
        request
            .extensions_mut()
            .insert(super::super::AuthenticatedLocalPeer);
        let response = router.oneshot(request).await.expect("route responds");
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(
            store
                .list_agent_profiles(OWNER, WORKSPACE, NOW)
                .expect("existing profile remains readable")
                .len(),
            1,
            "failed refresh must not replace the observed profile"
        );
    }

    fn seed_runtime(store: &super::super::SqliteWorkspaceStore) {
        use storage_core::{
            DeviceIdentityRecord, RuntimeIncarnationLocalObservationRecord,
            RuntimeIncarnationRecord, RuntimeLifecycleStore, RuntimeRecord,
        };
        let public_key = [23_u8; 32];
        store
            .register_local_incarnation(
                RuntimeRecord {
                    runtime_id: "agent-control-runtime".to_owned(),
                    device_identity: DeviceIdentityRecord {
                        device_id: format!(
                            "ed25519-sha256:{}",
                            hex::encode(<sha2::Sha256 as sha2::Digest>::digest(public_key))
                        ),
                        public_key: format!("ed25519:{}", hex::encode(public_key)),
                        key_version: 1,
                        issued_at: NOW.to_owned(),
                        display_name: None,
                    },
                    runtime_version: "test".to_owned(),
                    platform: "linux".to_owned(),
                    architecture: "x86_64".to_owned(),
                    roles: vec!["OPERATOR_ENDPOINT".to_owned()],
                    trust_zone: "PERSONAL_DEVICE".to_owned(),
                    availability: "RECOVERING".to_owned(),
                    startup_policy: "MANUAL".to_owned(),
                    current_incarnation_id: "agent-control-incarnation".to_owned(),
                    resource_capacity: serde_json::json!({"sampled_at":NOW}),
                    last_seen: NOW.to_owned(),
                },
                RuntimeIncarnationRecord {
                    runtime_incarnation_id: "agent-control-incarnation".to_owned(),
                    runtime_id: "agent-control-runtime".to_owned(),
                    process_started_at: NOW.to_owned(),
                    litecowork_version: "test".to_owned(),
                    recovered_from_unclean_shutdown: false,
                    recovery_state: "RECOVERING".to_owned(),
                    ready_at: None,
                    stopped_at: None,
                    version: 1,
                },
                RuntimeIncarnationLocalObservationRecord {
                    runtime_incarnation_id: "agent-control-incarnation".to_owned(),
                    os_boot_id: Some("agent-control-boot".to_owned()),
                    observed_at: NOW.to_owned(),
                },
            )
            .expect("Runtime incarnation registers");
    }

    fn seed_opencode_profile(store: &super::super::SqliteWorkspaceStore) {
        store
            .put_agent_profile(
                AgentProfileRecord {
                    agent_profile_id: "profile-opencode".to_owned(),
                    provider_key: "OPENCODE".to_owned(),
                    display_name: "OpenCode".to_owned(),
                    discovered_at: NOW.to_owned(),
                },
                vec![AgentEndpointRecord {
                    endpoint_id: "endpoint-opencode".to_owned(),
                    agent_profile_id: "profile-opencode".to_owned(),
                    protocol: "API".to_owned(),
                    topology: "PROCESS_ADAPTER".to_owned(),
                    protocol_version: Some("2.0.32".to_owned()),
                    capabilities: serde_json::json!({"input.text":true}),
                }],
            )
            .expect("OpenCode profile persists");
    }

    fn opencode_router(
        store: super::super::SqliteWorkspaceStore,
        version: Option<&'static str>,
    ) -> Router {
        let service = Arc::new(AgentDescriptorService::new(
            crate::agents::modules::default_module_registry(),
            Arc::new(crate::agents::descriptor_service::tests::FixtureProbe(
                version,
            )),
        ));
        test_router_with(store, OWNER.to_owned(), service)
    }

    async fn refresh_opencode(router: &Router) {
        let response = post(router, "/v1/agent-profiles/profile-opencode/refresh").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    async fn error_code(response: Response) -> String {
        let body = to_bytes(response.into_body(), 8192).await.expect("body");
        let value: serde_json::Value = serde_json::from_slice(&body).expect("JSON error");
        value["code"].as_str().unwrap_or_default().to_owned()
    }

    fn opencode_service() -> Arc<AgentDescriptorService> {
        Arc::new(AgentDescriptorService::new(
            crate::agents::modules::default_module_registry(),
            Arc::new(crate::agents::descriptor_service::tests::FixtureProbe(
                Some("2.0.32"),
            )),
        ))
    }

    #[tokio::test]
    async fn agent_auth_begin_rejects_an_undeclared_method_and_never_launches() {
        let (_directory, store) = fixture();
        seed_runtime(&store);
        seed_opencode_profile(&store);
        let control = opencode_router(store.clone(), Some("2.0.32"));
        refresh_opencode(&control).await;
        let router = super::super::agent_auth_operator::test_router(
            AgentControlApiState::for_test(store, OWNER.to_owned(), opencode_service()),
        );
        let response = post(
            &router,
            "/v1/agent-profiles/profile-opencode/auth/native-login/begin",
        )
        .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(error_code(response).await, "INVALID_ARGUMENT");
    }

    #[tokio::test]
    async fn agent_auth_begin_without_a_fresh_descriptor_or_module_is_unavailable() {
        let (_directory, store) = fixture();
        seed_runtime(&store);
        seed_opencode_profile(&store);
        seed_profile(&store);
        let router = super::super::agent_auth_operator::test_router(
            AgentControlApiState::for_test(store, OWNER.to_owned(), opencode_service()),
        );
        // No refresh yet: stale or missing descriptor never authorises a launch.
        let missing = post(
            &router,
            "/v1/agent-profiles/profile-opencode/auth/native-login/begin",
        )
        .await;
        assert_eq!(missing.status(), StatusCode::SERVICE_UNAVAILABLE);
        let codex = post(
            &router,
            "/v1/agent-profiles/profile-control/auth/native-login/begin",
        )
        .await;
        assert_eq!(codex.status(), StatusCode::SERVICE_UNAVAILABLE);
        let bad = post(
            &router,
            "/v1/agent-profiles/profile-opencode/auth/%00/begin",
        )
        .await;
        assert_eq!(bad.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn native_config_open_rejects_undeclared_targets_and_caller_paths() {
        let (_directory, store) = fixture();
        seed_runtime(&store);
        seed_opencode_profile(&store);
        let control = opencode_router(store.clone(), Some("2.0.32"));
        refresh_opencode(&control).await;
        let router = super::super::agent_native_config_operator::test_router(
            AgentControlApiState::for_test(store, OWNER.to_owned(), opencode_service()),
        );
        // OpenCode declares no native configuration target in V1.
        let undeclared = post(
            &router,
            "/v1/agent-profiles/profile-opencode/native-config/settings/open",
        )
        .await;
        assert_eq!(undeclared.status(), StatusCode::BAD_REQUEST);
        assert_eq!(error_code(undeclared).await, "INVALID_ARGUMENT");
        let traversal = post(
            &router,
            "/v1/agent-profiles/profile-opencode/native-config/..%2Fetc/open",
        )
        .await;
        assert_eq!(traversal.status(), StatusCode::BAD_REQUEST);
    }

    async fn post(router: &Router, path: &str) -> Response {
        let mut request = Request::post(path)
            .header("x-workspace-id", WORKSPACE)
            .body(Body::empty())
            .expect("request is valid");
        request
            .extensions_mut()
            .insert(super::super::AuthenticatedLocalPeer);
        router
            .clone()
            .oneshot(request)
            .await
            .expect("route responds")
    }

    #[tokio::test]
    async fn opencode_profile_serves_a_validated_descriptor_after_refresh_and_never_before() {
        let (_directory, store) = fixture();
        seed_runtime(&store);
        seed_opencode_profile(&store);
        let router = opencode_router(store, Some("2.0.32"));
        let path = "/v1/agent-profiles/profile-opencode/control-descriptor";

        // Nothing is manufactured before a refresh.
        assert_eq!(
            request(&router, path).await.status(),
            StatusCode::SERVICE_UNAVAILABLE
        );

        let refreshed = post(&router, "/v1/agent-profiles/profile-opencode/refresh").await;
        assert_eq!(refreshed.status(), StatusCode::OK);
        assert_eq!(refreshed.headers()[header::CACHE_CONTROL], "no-store");
        let refreshed_body: serde_json::Value = serde_json::from_slice(
            &to_bytes(refreshed.into_body(), 262_144)
                .await
                .expect("body"),
        )
        .expect("descriptor JSON");

        let served = request(&router, path).await;
        assert_eq!(served.status(), StatusCode::OK);
        assert_eq!(served.headers()[header::CACHE_CONTROL], "no-store");
        let served_body: serde_json::Value =
            serde_json::from_slice(&to_bytes(served.into_body(), 262_144).await.expect("body"))
                .expect("descriptor JSON");
        assert_eq!(
            served_body["descriptor_digest"],
            refreshed_body["descriptor_digest"]
        );
        assert_eq!(served_body["module_id"], "opencode");
        assert_eq!(served_body["auth"]["state"], "UNKNOWN");
        let text = served_body.to_string();
        assert!(!text.contains("endpoint-opencode") && !text.contains("/usr"));
    }

    #[tokio::test]
    async fn opencode_refresh_over_budget_returns_a_typed_throttle_and_keeps_reads_working() {
        let (_directory, store) = fixture();
        seed_runtime(&store);
        seed_opencode_profile(&store);
        let router = opencode_router(store, Some("2.0.32"));
        for _ in 0..3 {
            let response = post(&router, "/v1/agent-profiles/profile-opencode/refresh").await;
            assert_eq!(response.status(), StatusCode::OK);
        }
        let throttled = post(&router, "/v1/agent-profiles/profile-opencode/refresh").await;
        assert_eq!(throttled.status(), StatusCode::TOO_MANY_REQUESTS);
        let body = to_bytes(throttled.into_body(), 8192).await.expect("body");
        let value: serde_json::Value = serde_json::from_slice(&body).expect("JSON error");
        assert_eq!(value["code"], "RATE_LIMITED");
        // The stored descriptor is still readable after a refused refresh.
        let served = request(
            &router,
            "/v1/agent-profiles/profile-opencode/control-descriptor",
        )
        .await;
        assert_eq!(served.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn opencode_refresh_for_an_unpinned_version_stores_nothing() {
        let (_directory, store) = fixture();
        seed_runtime(&store);
        seed_opencode_profile(&store);
        let router = opencode_router(store, Some("2.0.28"));
        let response = post(&router, "/v1/agent-profiles/profile-opencode/refresh").await;
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let served = request(
            &router,
            "/v1/agent-profiles/profile-opencode/control-descriptor",
        )
        .await;
        assert_eq!(served.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
