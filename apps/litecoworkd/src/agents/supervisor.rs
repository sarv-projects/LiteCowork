//! AgentSessionSupervisor and AgentTurnCoordinator for Conversation turns.
//!
//! Orchestrates the three-phase session lifecycle:
//! 1. Reservation: STARTING AgentSession committed in SQLite.
//! 2. Native Handshake: Spawn or connect to native agent adapter (OpenCode, Claude Code, etc.)
//!    outside SQLite.
//! 3. Activation: Atomic SQLite CAS `activate_conversation_agent_session` binding native handle and
//!    transitioning turn OPEN -> RUNNING.
//! 4. Prompt Dispatch: Canonical conversation context package materialized and sent to native agent.
//! 5. Response Streaming & Terminal Settlement: Stream deltas collected and atomically committed
//!    via `commit_conversation_agent_response`.

use std::path::PathBuf;

use serde_json::json;
use storage_core::conversation::{
    CommitConversationAgentResponse, ConversationMessageRecord, ConversationTurn,
    ConversationTurnStatus, ConversationTurnStore, TurnCommand,
};
use storage_core::{
    ActivateConversationAgentSession, AgentCatalogStore, AgentHostInstanceRecord, AgentHostStore,
    AgentSessionRecord, AgentSessionStore, ConversationAgentSessionStart, EventDraft,
    MarkStartingConversationAgentSessionLost, WorkspaceCreateRequest,
};
use storage_sqlite::{SqliteConversationStore, SqliteWorkspaceStore};

use super::adapter::{AgentAdapter, OpenCodeAdapter, StreamBudget, TurnObservation};
use super::native_provider::NativeProviderKey;
use super::opencode_server::{
    OpenCodeEnvironment, OpenCodeServer, OpenCodeServerConfig, OpenCodeServerProtocol,
};

fn now_utc_rfc3339() -> String {
    let now = time::OffsetDateTime::now_utc();
    now.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                now.year(),
                now.month() as u8,
                now.day(),
                now.hour(),
                now.minute(),
                now.second()
            )
        })
}

fn generate_id(prefix: &str) -> String {
    let mut bytes = [0_u8; 16];
    if let Err(e) = getrandom::fill(&mut bytes) {
        eprintln!("WARNING: getrandom failed in generate_id: {e:?}");
        let nanos = time::OffsetDateTime::now_utc().unix_timestamp_nanos();
        return format!("{}_{:x}", prefix, nanos);
    }
    format!("{}_{}", prefix, hex::encode(bytes))
}

fn resolve_workspace_working_dir(
    store: &SqliteWorkspaceStore,
    principal_id: &str,
    workspace_id: &str,
    runtime_id: &str,
) -> Option<PathBuf> {
    use storage_core::{WorkspaceRootRevalidationBindings, WorkspaceRootStore};
    if let Ok(roots) = store.list_workspace_roots(workspace_id, Some("ACTIVE"), None, None, 1)
        && let Some(first) = roots.first()
        && let Ok(Some(candidate)) = store.get_workspace_root_scan_candidate(
            principal_id,
            workspace_id,
            &first.root.workspace_root_id,
            runtime_id,
            "default",
        )
    {
        match candidate.bindings {
            WorkspaceRootRevalidationBindings::CurrentIncarnation { locator, .. }
            | WorkspaceRootRevalidationBindings::Previous { locator, .. } => {
                // `private_locator` is an opaque encoded locator (for example
                // `unix-path-b64:`), never a filesystem path. Building a PathBuf
                // from the encoded string can never name an existing directory,
                // so the Workspace root was silently ignored and the agent child
                // ran in the daemon's own working directory instead.
                if let Some(path) = crate::local_filesystem::decode_workspace_root_locator_path(
                    &locator.private_locator,
                ) && path.is_dir()
                {
                    return Some(path);
                }
            }
            _ => {}
        }
    }
    // Fail closed. A turn that cannot resolve its own Workspace root must not run an
    // agent in an unrelated directory: the daemon's current directory is not an
    // authorization the user granted, and silently using it hid a decoding defect.
    None
}

#[derive(Clone)]
pub struct AgentSessionSupervisor {
    store: SqliteWorkspaceStore,
    runtime_id: String,
}

impl AgentSessionSupervisor {
    pub fn new(store: SqliteWorkspaceStore, runtime_id: String) -> Self {
        Self { store, runtime_id }
    }

    /// Dispatches a submitted ConversationTurn through the full native agent lifecycle.
    pub async fn dispatch_turn(
        &self,
        principal_id: &str,
        workspace_id: &str,
        conversation_id: &str,
        turn: ConversationTurn,
        correlation_id: &str,
    ) -> Result<(), String> {
        let store = self.store.clone();
        let runtime_id = self.runtime_id.clone();
        let principal = principal_id.to_owned();
        let workspace = workspace_id.to_owned();
        let conversation = conversation_id.to_owned();
        let turn_id = turn.turn_id.clone();
        let correlation = correlation_id.to_owned();

        tokio::task::spawn_blocking(move || {
            let conv_store = SqliteConversationStore::new(store.clone());

            // 1. Resolve host for turn
            let host_opt = conv_store
                .resolve_conversation_session_host_for_turn(
                    &principal,
                    &workspace,
                    &conversation,
                    &turn_id,
                )
                .map_err(|e| format!("Host resolution error: {e:?}"))?;

            let Some(host) = host_opt else {
                // Settle turn FAILED: AGENT_UNAVAILABLE
                let now = now_utc_rfc3339();
                let settled_event = EventDraft {
                    event_id: generate_id("ev"),
                    workspace_id: workspace.clone(),
                    entity_type: "ConversationTurn".to_owned(),
                    entity_id: turn_id.clone(),
                    origin_runtime_id: runtime_id.clone(),
                    entity_revision: turn.version + 1,
                    hlc_timestamp: now.clone(),
                    correlation_id: correlation.clone(),
                    causation_id: None,
                    schema_version: 1,
                    event_type: "conversation.turn.settled.v1".to_owned(),
                    payload: json!({
                        "turn_id": turn_id,
                        "from": "OPEN",
                        "to": "FAILED",
                        "reason_code": "AGENT_UNAVAILABLE",
                        "aggregate_version": turn.version + 1,
                    }),
                    recorded_at: now.clone(),
                };
                let _ = conv_store.transition_conversation_turn(
                    &principal,
                    &workspace,
                    &conversation,
                    &turn_id,
                    turn.version,
                    TurnCommand::Fail { settled_at: now },
                    settled_event,
                );
                return Ok(());
            };

            // 2. Query provider key and profile via published AgentCatalogStore trait
            let now = now_utc_rfc3339();
            let binding = store
                .get_agent_binding(&principal, &workspace, &host.agent_binding_id)
                .map_err(|e| format!("Query agent binding error: {e:?}"))?
                .ok_or_else(|| "AgentBinding not found".to_owned())?;

            let profiles = store
                .list_agent_profiles(&principal, &workspace, &now)
                .map_err(|e| format!("List agent profiles error: {e:?}"))?;

            let provider_key = profiles
                .iter()
                .find(|p| p.profile.agent_profile_id == binding.agent_profile_id)
                .map(|p| p.profile.provider_key.clone())
                .ok_or_else(|| {
                    format!(
                        "No agent profile found for binding {} with profile ID {}",
                        binding.agent_binding_id, binding.agent_profile_id
                    )
                })?;

            // 3. Ensure host instance exists in SQLite via published AgentHostStore trait
            let host_instance_id = format!("host-{}", host.endpoint_id);
            let existing_host = store
                .get_agent_host_instance(
                    &host.runtime_id,
                    &host.runtime_incarnation_id,
                    &host_instance_id,
                )
                .map_err(|e| format!("Get host instance error: {e:?}"))?;

            if existing_host.is_none() {
                let _ = store.create_agent_host_instance(AgentHostInstanceRecord {
                    host_instance_id: host_instance_id.clone(),
                    runtime_id: host.runtime_id.clone(),
                    runtime_incarnation_id: host.runtime_incarnation_id.clone(),
                    agent_profile_id: binding.agent_profile_id.clone(),
                    endpoint_id: host.endpoint_id.clone(),
                    hosting_mode: "LOCAL_PER_SESSION".to_owned(),
                    state: "READY".to_owned(),
                    process_identity_ref: None,
                    ownership: "LITECOWORK".to_owned(),
                    started_at: now.clone(),
                    last_used_at: now.clone(),
                    idle_since: Some(now.clone()),
                });
            }

            // 3b. Claim the durable dispatch intent (compare-and-swap) before the STARTING
            // reservation. A losing dispatcher owns nothing and must not reserve a session.
            let claimed = conv_store
                .claim_conversation_dispatch_intent(
                    &turn_id,
                    turn.retry_ordinal,
                    &host.runtime_incarnation_id,
                    &now,
                )
                .map_err(|e| format!("Claim dispatch intent error: {e:?}"))?;
            if !claimed {
                return Ok(());
            }

            // 4. Reserve STARTING session
            let session_id = generate_id("ses");
            let start = ConversationAgentSessionStart {
                request: WorkspaceCreateRequest {
                    principal_id: principal.clone(),
                    request_id: format!("req-start-{}", session_id),
                    request_payload: json!({ "agent_session_id": session_id }),
                },
                session: AgentSessionRecord {
                    agent_session_id: session_id.clone(),
                    workspace_id: workspace.clone(),
                    scope_kind: "CONVERSATION".to_owned(),
                    conversation_id: Some(conversation.clone()),
                    conversation_turn_id: Some(turn_id.clone()),
                    task_id: None,
                    task_spec_revision: None,
                    attempt_id: None,
                    agent_binding_id: host.agent_binding_id.clone(),
                    endpoint_id: host.endpoint_id.clone(),
                    runtime_id: host.runtime_id.clone(),
                    runtime_incarnation_id: host.runtime_incarnation_id.clone(),
                    configuration_digest: None,
                    harness_descriptor_digest: None,
                    status: "STARTING".to_owned(),
                    started_at: now.clone(),
                    last_event_at: Some(now.clone()),
                    closed_at: None,
                    version: 1,
                },
                event: EventDraft {
                    event_id: generate_id("ev"),
                    workspace_id: workspace.clone(),
                    entity_type: "AgentSession".to_owned(),
                    entity_id: session_id.clone(),
                    origin_runtime_id: runtime_id.clone(),
                    entity_revision: 1,
                    hlc_timestamp: now.clone(),
                    correlation_id: correlation.clone(),
                    causation_id: None,
                    schema_version: 1,
                    event_type: "agent.session.starting.v1".to_owned(),
                    payload: json!({
                        "agent_session_id": session_id,
                        "task_spec_revision": null,
                        "scope": {
                            "kind": "CONVERSATION",
                            "conversation_id": conversation,
                            "conversation_turn_id": turn_id,
                        },
                        "agent_binding_id": host.agent_binding_id,
                        "endpoint_id": host.endpoint_id,
                        "runtime_id": host.runtime_id,
                        "runtime_incarnation_id": host.runtime_incarnation_id,
                        "session_state": "STARTING",
                        "reported_at": now,
                    }),
                    recorded_at: now.clone(),
                },
            };

            let _committed_session = store
                .start_conversation_agent_session(start)
                .map_err(|e| format!("Start session error: {e:?}"))?;

            // 5. Execute native interaction based on provider_key
            let run_res = execute_agent_turn(
                &store,
                &runtime_id,
                &principal,
                &workspace,
                &conversation,
                &turn_id,
                turn.version,
                turn.retry_ordinal,
                &session_id,
                &host_instance_id,
                &host.agent_binding_id,
                &provider_key,
                &host.endpoint_id,
                &host.runtime_id,
                &host.runtime_incarnation_id,
                &correlation,
            );

            if let Err(err) = run_res {
                let now = now_utc_rfc3339();
                // Check if turn was already activated (RUNNING) or still OPEN
                let current_turn = conv_store
                    .get_conversation_turn(&workspace, &conversation, &turn_id)
                    .ok()
                    .flatten();
                let current_version = current_turn
                    .as_ref()
                    .map(|t| t.version)
                    .unwrap_or(turn.version);
                let current_status_str = current_turn
                    .as_ref()
                    .map(|t| match t.status {
                        ConversationTurnStatus::Running => "RUNNING",
                        _ => "OPEN",
                    })
                    .unwrap_or("OPEN");

                let settled_event = EventDraft {
                    event_id: generate_id("ev"),
                    workspace_id: workspace.clone(),
                    entity_type: "ConversationTurn".to_owned(),
                    entity_id: turn_id.clone(),
                    origin_runtime_id: runtime_id.clone(),
                    entity_revision: current_version + 1,
                    hlc_timestamp: now.clone(),
                    correlation_id: correlation.clone(),
                    causation_id: None,
                    schema_version: 1,
                    event_type: "conversation.turn.settled.v1".to_owned(),
                    payload: json!({
                        "turn_id": turn_id,
                        "from": current_status_str,
                        "to": "FAILED",
                        "reason_code": "CONVERSATION_SESSION_ADMISSION_UNAVAILABLE",
                        "aggregate_version": current_version + 1,
                    }),
                    recorded_at: now.clone(),
                };
                let _ = conv_store.transition_conversation_turn(
                    &principal,
                    &workspace,
                    &conversation,
                    &turn_id,
                    current_version,
                    TurnCommand::Fail {
                        settled_at: now.clone(),
                    },
                    settled_event,
                );

                if current_status_str == "OPEN" {
                    let _ = store.mark_starting_conversation_agent_session_lost(
                        MarkStartingConversationAgentSessionLost {
                            workspace_id: workspace.clone(),
                            agent_session_id: session_id.clone(),
                            expected_version: 1,
                            occurred_at: now.clone(),
                            event: EventDraft {
                                event_id: generate_id("ev"),
                                workspace_id: workspace.clone(),
                                entity_type: "AgentSession".to_owned(),
                                entity_id: session_id.clone(),
                                origin_runtime_id: runtime_id.clone(),
                                entity_revision: 2,
                                hlc_timestamp: now.clone(),
                                correlation_id: correlation.clone(),
                                causation_id: None,
                                schema_version: 1,
                                event_type: "agent.session.lost.v1".to_owned(),
                                payload: json!({
                                    "agent_session_id": session_id,
                                    "from": "STARTING",
                                    "to": "LOST",
                                    "reason_code": "NATIVE_SPAWN_FAILED",
                                }),
                                recorded_at: now.clone(),
                            },
                        },
                    );
                }
                return Err(err);
            }

            Ok(())
        })
        .await
        .map_err(|e| format!("Task execution error: {e:?}"))?
    }
}

// Twelve distinct turn-identity inputs; a parameter struct would only relabel
// them at this executor's single call site in this module.
#[allow(clippy::too_many_arguments)]
fn execute_agent_turn(
    store: &SqliteWorkspaceStore,
    runtime_id: &str,
    principal_id: &str,
    workspace_id: &str,
    conversation_id: &str,
    turn_id: &str,
    turn_version: u64,
    retry_ordinal: u32,
    session_id: &str,
    host_instance_id: &str,
    agent_binding_id: &str,
    provider_key: &str,
    endpoint_id: &str,
    host_runtime_id: &str,
    host_runtime_incarnation_id: &str,
    correlation_id: &str,
) -> Result<(), String> {
    let conv_store = SqliteConversationStore::new(store.clone());

    // Parse the stored provider key once at the boundary. The canonical stored
    // form is upper case (`OPENCODE`); an unknown key fails closed below.
    let provider = NativeProviderKey::from_profile_key(provider_key);

    // Native session establishment
    match provider {
        Some(NativeProviderKey::OpenCode) => {
            // Dispatch exactly the executable the probe registered for this
            // endpoint and re-validate it is the same canonical path. Probe or
            // dispatch executable drift fails the turn closed with no native spawn.
            let now = now_utc_rfc3339();
            let binding = store
                .get_local_endpoint_binding(
                    host_runtime_id,
                    host_runtime_incarnation_id,
                    endpoint_id,
                    &now,
                )
                .map_err(|e| format!("Query endpoint binding error: {e:?}"))?
                .ok_or_else(|| {
                    "no fresh Runtime-local endpoint binding for the OpenCode endpoint".to_owned()
                })?;
            let registered = PathBuf::from(&binding.endpoint_ref);
            let canonical_registered = std::fs::canonicalize(&registered).map_err(|_| {
                "registered OpenCode endpoint executable no longer resolves".to_owned()
            })?;
            if !canonical_registered.is_file() {
                return Err("registered OpenCode endpoint executable is not a file".to_owned());
            }
            // Re-resolve through the shared resolver and require the same canonical
            // path, so a changed PATH or a replaced executable fails closed.
            let resolved = crate::agents::resolve_opencode_executable()
                .ok_or_else(|| "opencode executable not found".to_owned())?;
            if resolved != canonical_registered {
                return Err(
                    "OpenCode executable drifted from the registered endpoint reference".to_owned(),
                );
            }
            let exe = canonical_registered;
            let working_dir =
                resolve_workspace_working_dir(store, principal_id, workspace_id, runtime_id)
                    .ok_or_else(|| {
                        "no active Workspace root resolves to an existing directory".to_owned()
                    })?;
            // The child is given the same bounded variable allow list that the adapter
            // clears the environment to, sourced from this process. Without it the agent
            // child starts with no HOME and no PATH and cannot run at all.
            let config = OpenCodeServerConfig::bounded(exe, working_dir)
                .with_protocol(OpenCodeServerProtocol::V2_0_32)
                .with_environment(OpenCodeEnvironment::from_current_process());

            let server = OpenCodeServer::spawn(config)
                .map_err(|e| format!("OpenCodeServer spawn failed: {e:?}"))?;

            // Drive the native boundary through the provider-neutral AgentAdapter
            // (001-T04) rather than hand-parsing SSE. The OpenCode v2 HTTP path is
            // the one non-ACP adapter; it owns the server for the life of the turn.
            let mut adapter = OpenCodeAdapter::new(server);

            // Phase 1: version/capability probe (B-03). A failed probe is a typed
            // FAILED turn with the USER message kept and no fallback (AC-015).
            adapter.probe_version().map_err(|f| {
                format!("OpenCode adapter version probe failed: {}", f.reason_code())
            })?;

            // Phase 2: create the native session; the handle binds to SQLite only.
            let native_session = adapter
                .create_session()
                .map_err(|f| format!("OpenCode create_session failed: {}", f.reason_code()))?;

            // Activate session in SQLite
            let now = now_utc_rfc3339();
            let activation = ActivateConversationAgentSession {
                principal_id: principal_id.to_owned(),
                workspace_id: workspace_id.to_owned(),
                agent_session_id: session_id.to_owned(),
                expected_session_version: 1,
                expected_turn_version: turn_version,
                occurred_at: now.clone(),
                host_instance_id: host_instance_id.to_owned(),
                native_session_ref: Some(native_session.clone()),
                session_event: EventDraft {
                    event_id: generate_id("ev"),
                    workspace_id: workspace_id.to_owned(),
                    entity_type: "AgentSession".to_owned(),
                    entity_id: session_id.to_owned(),
                    origin_runtime_id: runtime_id.to_owned(),
                    entity_revision: 2,
                    hlc_timestamp: now.clone(),
                    correlation_id: correlation_id.to_owned(),
                    causation_id: None,
                    schema_version: 1,
                    event_type: "agent.session.started.v1".to_owned(),
                    payload: json!({
                        "agent_session_id": session_id,
                        "task_spec_revision": null,
                        "scope": {
                            "kind": "CONVERSATION",
                            "conversation_id": conversation_id,
                            "conversation_turn_id": turn_id,
                        },
                        "agent_binding_id": agent_binding_id,
                        "session_state": "ACTIVE",
                        "reported_at": now,
                    }),
                    recorded_at: now.clone(),
                },
                turn_event: EventDraft {
                    event_id: generate_id("ev"),
                    workspace_id: workspace_id.to_owned(),
                    entity_type: "ConversationTurn".to_owned(),
                    entity_id: turn_id.to_owned(),
                    origin_runtime_id: runtime_id.to_owned(),
                    entity_revision: turn_version + 1,
                    hlc_timestamp: now.clone(),
                    correlation_id: correlation_id.to_owned(),
                    causation_id: None,
                    schema_version: 1,
                    event_type: "conversation.turn.status.changed.v1".to_owned(),
                    payload: json!({
                        "turn_id": turn_id,
                        "from": "OPEN",
                        "to": "RUNNING",
                        "reason_code": "SESSION_READY",
                        "agent_session_id": session_id,
                        "aggregate_version": turn_version + 1,
                    }),
                    recorded_at: now.clone(),
                },
            };

            let activated = store
                .activate_conversation_agent_session(activation)
                .map_err(|e| format!("Activate session error: {e:?}"))?;

            // Load context package
            let context = conv_store
                .load_conversation_context_for_turn(
                    principal_id,
                    workspace_id,
                    conversation_id,
                    turn_id,
                )
                .map_err(|e| format!("Load context error: {e:?}"))?;

            // Phase 3: dispatch the bounded committed context only after the
            // activation commit above. The native message id correlates native
            // events; it is not treated as an idempotency guarantee.
            let native_msg_id = format!("msg_{}", generate_id("m"));
            adapter
                .send_context(&native_session, &native_msg_id, &context)
                .map_err(|f| format!("Send context error: {}", f.reason_code()))?;

            // Phase 4: consume the correlated, bounded event stream through the
            // normalizer under a line-resetting idle/heartbeat watchdog (replaces
            // the old hard 60 s cap). Every event is correlated to the native
            // session; uncorrelated, foreign-session, oversized and unknown frames
            // are dropped and counted, never projected (AC-008).
            let observation = adapter.drive_turn(&native_session, StreamBudget::default());

            // The server is reaped regardless of outcome.
            adapter.shutdown();

            let accumulated_text = match observation {
                Ok(TurnObservation::Completed { answer_text }) => answer_text,
                Ok(TurnObservation::Stopped) => {
                    // A stop observation is not a settlement; the caller's error
                    // path reconciles it through session state, never a resend.
                    return Err("OpenCode turn stopped before completion".to_owned());
                }
                Ok(TurnObservation::Failed { reason_code }) | Err(reason_code) => {
                    return Err(format!(
                        "OpenCode turn failed closed: {}",
                        reason_code.reason_code()
                    ));
                }
            };

            // The commit identity is deterministic in (turn, retry ordinal, native
            // message) so a repeated commit attempt replays the stored receipt instead
            // of writing a second answer (B-04).
            let final_message_id = format!("msg-agent-{turn_id}-{retry_ordinal}");
            let provider_response_id = format!("{native_session}/{native_msg_id}");
            let now = now_utc_rfc3339();
            let commit_resp = CommitConversationAgentResponse {
                request: WorkspaceCreateRequest {
                    principal_id: principal_id.to_owned(),
                    request_id: format!("req-commit-{turn_id}-{retry_ordinal}"),
                    request_payload: json!({
                        "turn_id": turn_id,
                        "retry_ordinal": retry_ordinal,
                        "provider_response_id": provider_response_id,
                    }),
                },
                principal_id: principal_id.to_owned(),
                workspace_id: workspace_id.to_owned(),
                conversation_id: conversation_id.to_owned(),
                turn_id: turn_id.to_owned(),
                agent_session_id: session_id.to_owned(),
                expected_turn_version: activated.turn.version,
                expected_session_version: activated.session.version,
                retry_ordinal,
                provider_response_id,
                usage: None,
                turn_status: ConversationTurnStatus::Completed,
                message: ConversationMessageRecord {
                    message_id: final_message_id.clone(),
                    conversation_id: conversation_id.to_owned(),
                    author: json!({ "kind": "AGENT", "agent_binding_id": agent_binding_id }),
                    role: "AGENT".to_owned(),
                    agent_session_id: Some(session_id.to_owned()),
                    agent_binding_id: Some(agent_binding_id.to_owned()),
                    turn_id: Some(turn_id.to_owned()),
                    content: json!([{ "kind": "TEXT", "text": accumulated_text }]),
                    resource_refs: json!([]),
                    source_channel_ref: None,
                    created_at: now.clone(),
                },
                message_event: EventDraft {
                    event_id: generate_id("ev"),
                    workspace_id: workspace_id.to_owned(),
                    entity_type: "Conversation".to_owned(),
                    entity_id: conversation_id.to_owned(),
                    origin_runtime_id: runtime_id.to_owned(),
                    entity_revision: 3,
                    hlc_timestamp: now.clone(),
                    correlation_id: correlation_id.to_owned(),
                    causation_id: None,
                    schema_version: 1,
                    event_type: "conversation.message.added.v1".to_owned(),
                    payload: json!({
                        "conversation_id": conversation_id,
                        "message_id": final_message_id,
                        "role": "AGENT",
                    }),
                    recorded_at: now.clone(),
                },
                turn_event: EventDraft {
                    event_id: generate_id("ev"),
                    workspace_id: workspace_id.to_owned(),
                    entity_type: "ConversationTurn".to_owned(),
                    entity_id: turn_id.to_owned(),
                    origin_runtime_id: runtime_id.to_owned(),
                    entity_revision: activated.turn.version + 1,
                    hlc_timestamp: now.clone(),
                    correlation_id: correlation_id.to_owned(),
                    causation_id: None,
                    schema_version: 1,
                    event_type: "conversation.turn.settled.v1".to_owned(),
                    payload: json!({
                        "turn_id": turn_id,
                        "from": "RUNNING",
                        "to": "COMPLETED",
                        "aggregate_version": activated.turn.version + 1,
                    }),
                    recorded_at: now.clone(),
                },
                session_event: EventDraft {
                    event_id: generate_id("ev"),
                    workspace_id: workspace_id.to_owned(),
                    entity_type: "AgentSession".to_owned(),
                    entity_id: session_id.to_owned(),
                    origin_runtime_id: runtime_id.to_owned(),
                    entity_revision: activated.session.version + 1,
                    hlc_timestamp: now.clone(),
                    correlation_id: correlation_id.to_owned(),
                    causation_id: None,
                    schema_version: 1,
                    event_type: "agent.session.closed.v1".to_owned(),
                    payload: json!({
                        "agent_session_id": session_id,
                        "from": "ACTIVE",
                        "to": "CLOSED",
                        "aggregate_version": activated.session.version + 1,
                    }),
                    recorded_at: now.clone(),
                },
            };

            conv_store
                .commit_conversation_agent_response(commit_resp)
                .map_err(|e| format!("Commit response error: {e:?}"))?;

            Ok(())
        }
        _ => Err(format!(
            "Agent provider '{provider_key}' is not supported or not configured for execution in this runtime"
        )),
    }
}
