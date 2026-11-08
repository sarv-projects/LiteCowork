//! Transactional Environment persistence port and its request envelopes.
//!
//! Canonical Environment vocabulary, identity newtypes, and records are re-exported
//! from `domain-environment` (crate module `environment`) so existing `storage_core::`
//! paths keep resolving. This module owns the `EnvironmentStore` port, the idempotency
//! request envelopes, and the Runtime-local provider-binding commit shape that store
//! adapters execute against.

use crate::{EventDraft, StoreError};
pub use domain_environment::environment::*;

/// Vocabulary validation failures surface as invalid input with the original message
/// so adapters keep a single error taxonomy.
impl From<EnvironmentValidationError> for StoreError {
    fn from(error: EnvironmentValidationError) -> Self {
        StoreError::Invalid(error.message().to_owned())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnvironmentRequestIdentity {
    /// Authenticated Operator principal. The canonical body represents only the
    /// caller's public command and never includes provider locators, credentials, or
    /// provider receipt bytes.
    principal_id: PrincipalId,
    request_id: String,
    canonical_request_body: Vec<u8>,
    canonical_request_digest: String,
}

impl EnvironmentRequestIdentity {
    pub fn new(
        principal_id: PrincipalId,
        request_id: impl Into<String>,
        canonical_request_body: Vec<u8>,
        canonical_request_digest: impl Into<String>,
    ) -> Result<Self, StoreError> {
        let request_id = request_id.into();
        let canonical_request_digest = canonical_request_digest.into();
        if request_id.trim().is_empty() || canonical_request_body.is_empty() {
            return Err(StoreError::Invalid(
                "request identity requires request_id and canonical body".to_owned(),
            ));
        }
        if !valid_digest(&canonical_request_digest) {
            return Err(StoreError::Invalid(
                "canonical request digest must be a lowercase SHA-256 digest".to_owned(),
            ));
        }
        Ok(Self {
            principal_id,
            request_id,
            canonical_request_body,
            canonical_request_digest,
        })
    }

    pub fn principal_id(&self) -> &PrincipalId {
        &self.principal_id
    }
    pub fn request_id(&self) -> &str {
        &self.request_id
    }
    pub fn canonical_request_body(&self) -> &[u8] {
        &self.canonical_request_body
    }
    pub fn canonical_request_digest(&self) -> &str {
        &self.canonical_request_digest
    }
}

fn valid_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

/// Runtime-local opaque provider binding. It is written only to the private
/// `environment_provider_bindings` table in the same transaction as READY. Never
/// serialize this value into the request body, durable Environment aggregate, domain
/// event, backup, aggregate blob, or public projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderBindingCommit {
    environment_id: EnvironmentId,
    runtime_id: RuntimeId,
    runtime_incarnation_id: RuntimeIncarnationId,
    provider_kind: String,
    opaque_locator_ref: String,
    observed_at: String,
    expires_at: Option<String>,
}

impl ProviderBindingCommit {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        environment_id: EnvironmentId,
        runtime_id: RuntimeId,
        runtime_incarnation_id: RuntimeIncarnationId,
        provider_kind: impl Into<String>,
        opaque_locator_ref: impl Into<String>,
        observed_at: impl Into<String>,
        expires_at: Option<String>,
    ) -> Result<Self, StoreError> {
        let provider_kind = provider_kind.into();
        let opaque_locator_ref = opaque_locator_ref.into();
        let observed_at = observed_at.into();
        if provider_kind.trim().is_empty()
            || opaque_locator_ref.trim().is_empty()
            || observed_at.trim().is_empty()
            || expires_at
                .as_ref()
                .is_some_and(|timestamp| timestamp.trim().is_empty())
        {
            return Err(StoreError::Invalid(
                "provider binding fields must be non-empty".to_owned(),
            ));
        }
        Ok(Self {
            environment_id,
            runtime_id,
            runtime_incarnation_id,
            provider_kind,
            opaque_locator_ref,
            observed_at,
            expires_at,
        })
    }

    pub fn environment_id(&self) -> &EnvironmentId {
        &self.environment_id
    }
    pub fn runtime_id(&self) -> &RuntimeId {
        &self.runtime_id
    }
    pub fn runtime_incarnation_id(&self) -> &RuntimeIncarnationId {
        &self.runtime_incarnation_id
    }
    pub fn provider_kind(&self) -> &str {
        &self.provider_kind
    }
    pub fn opaque_locator_ref(&self) -> &str {
        &self.opaque_locator_ref
    }
    pub fn observed_at(&self) -> &str {
        &self.observed_at
    }
    pub fn expires_at(&self) -> Option<&str> {
        self.expires_at.as_deref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnvironmentCreateRequest {
    identity: EnvironmentRequestIdentity,
    record: EnvironmentRecord,
    event: EventDraft,
}

impl EnvironmentCreateRequest {
    /// Creates only V1-supported scoped Environment requests. Workspace-persistent
    /// creation requires a typed, validated preview-consumption transaction and is
    /// deliberately rejected until that contract is implemented.
    pub fn new(
        identity: EnvironmentRequestIdentity,
        record: EnvironmentRecord,
        event: EventDraft,
    ) -> Result<Self, StoreError> {
        if record.config().lifetime == EnvironmentLifetime::WorkspacePersistent {
            return Err(StoreError::Invalid(
                "WORKSPACE_PERSISTENT creation requires typed preview consumption, which is not implemented".to_owned(),
            ));
        }
        if record.status() != EnvironmentStatus::Provisioning {
            return Err(StoreError::Invalid(
                "Environment creation requires PROVISIONING state".to_owned(),
            ));
        }
        if record.updated_at() != event.recorded_at {
            return Err(StoreError::Invalid(
                "Environment updated_at must equal event recorded_at".to_owned(),
            ));
        }
        Ok(Self {
            identity,
            record,
            event,
        })
    }

    pub fn request_id(&self) -> &str {
        self.identity.request_id()
    }
    pub fn identity(&self) -> &EnvironmentRequestIdentity {
        &self.identity
    }
    pub fn record(&self) -> &EnvironmentRecord {
        &self.record
    }
    pub fn event(&self) -> &EventDraft {
        &self.event
    }
}

/// Stable-key Environment listing. `after_environment_id` is an exclusive cursor;
/// callers must supply an explicit positive limit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnvironmentListRequest {
    workspace_id: WorkspaceId,
    after_environment_id: Option<EnvironmentId>,
    limit: u32,
}

impl EnvironmentListRequest {
    pub fn new(
        workspace_id: WorkspaceId,
        after_environment_id: Option<EnvironmentId>,
        limit: u32,
    ) -> Result<Self, StoreError> {
        if limit == 0 {
            return Err(StoreError::Invalid(
                "Environment list limit must be positive".to_owned(),
            ));
        }
        Ok(Self {
            workspace_id,
            after_environment_id,
            limit,
        })
    }

    pub fn workspace_id(&self) -> &WorkspaceId {
        &self.workspace_id
    }
    pub fn after_environment_id(&self) -> Option<&EnvironmentId> {
        self.after_environment_id.as_ref()
    }
    pub const fn limit(&self) -> u32 {
        self.limit
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnvironmentLifecycleRequest {
    identity: EnvironmentRequestIdentity,
    environment_id: EnvironmentId,
    workspace_id: WorkspaceId,
    expected: ExpectedState,
    proposed: EnvironmentRecord,
    provider_binding: Option<ProviderBindingCommit>,
    event: EventDraft,
}

/// Explicit owner command for the F65 sharing-scope transition. The public request
/// digest covers the selected target and expected version; Runtime/provider data is
/// deliberately excluded.
#[derive(Clone, Debug, PartialEq)]
pub struct EnvironmentSharingScopeChangeRequest {
    identity: EnvironmentRequestIdentity,
    environment_id: EnvironmentId,
    workspace_id: WorkspaceId,
    expected_version: u64,
    target_scope: EnvironmentSharingScope,
    target_coworker_id: Option<CoworkerId>,
    event: EventDraft,
}

impl EnvironmentSharingScopeChangeRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        identity: EnvironmentRequestIdentity,
        environment_id: EnvironmentId,
        workspace_id: WorkspaceId,
        expected_version: u64,
        target_scope: EnvironmentSharingScope,
        target_coworker_id: Option<CoworkerId>,
        event: EventDraft,
    ) -> Result<Self, StoreError> {
        if expected_version == 0 || expected_version == u64::MAX {
            return Err(StoreError::Invalid(
                "Environment sharing-scope expected version is invalid".to_owned(),
            ));
        }
        match (target_scope, target_coworker_id.as_ref()) {
            (EnvironmentSharingScope::CoworkerPrivate, Some(_))
            | (EnvironmentSharingScope::WorkspaceShared, None) => {}
            _ => {
                return Err(StoreError::Invalid(
                    "Environment sharing-scope target is unavailable or has invalid owner"
                        .to_owned(),
                ));
            }
        }
        Ok(Self {
            identity,
            environment_id,
            workspace_id,
            expected_version,
            target_scope,
            target_coworker_id,
            event,
        })
    }

    pub fn identity(&self) -> &EnvironmentRequestIdentity {
        &self.identity
    }
    pub fn environment_id(&self) -> &EnvironmentId {
        &self.environment_id
    }
    pub fn workspace_id(&self) -> &WorkspaceId {
        &self.workspace_id
    }
    pub const fn expected_version(&self) -> u64 {
        self.expected_version
    }
    pub const fn target_scope(&self) -> EnvironmentSharingScope {
        self.target_scope
    }
    pub fn target_coworker_id(&self) -> Option<&CoworkerId> {
        self.target_coworker_id.as_ref()
    }
    pub fn event(&self) -> &EventDraft {
        &self.event
    }
}

impl EnvironmentLifecycleRequest {
    pub fn new(
        identity: EnvironmentRequestIdentity,
        environment_id: EnvironmentId,
        workspace_id: WorkspaceId,
        expected: ExpectedState,
        proposed: EnvironmentRecord,
        provider_binding: Option<ProviderBindingCommit>,
        event: EventDraft,
    ) -> Result<Self, StoreError> {
        if proposed.identity().environment_id != environment_id
            || proposed.identity().owner_workspace_id != workspace_id
        {
            return Err(StoreError::Invalid(
                "lifecycle request scope does not match proposed Environment".to_owned(),
            ));
        }
        if proposed.updated_at() != event.recorded_at {
            return Err(StoreError::Invalid(
                "Environment updated_at must equal event recorded_at".to_owned(),
            ));
        }
        if (proposed.status() == EnvironmentStatus::Ready) != provider_binding.is_some() {
            return Err(StoreError::Invalid(
                "READY transition requires a Runtime-local provider binding; other transitions must not supply one".to_owned(),
            ));
        }
        if let Some(binding) = &provider_binding
            && (binding.environment_id != environment_id
                || binding.runtime_id != proposed.identity().runtime_id
                || binding.provider_kind != proposed.config().provider_kind)
        {
            return Err(StoreError::Invalid(
                "provider binding does not match the Environment identity".to_owned(),
            ));
        }
        Ok(Self {
            identity,
            environment_id,
            workspace_id,
            expected,
            proposed,
            provider_binding,
            event,
        })
    }

    pub fn identity(&self) -> &EnvironmentRequestIdentity {
        &self.identity
    }
    pub fn environment_id(&self) -> &EnvironmentId {
        &self.environment_id
    }
    pub fn workspace_id(&self) -> &WorkspaceId {
        &self.workspace_id
    }
    pub const fn expected(&self) -> ExpectedState {
        self.expected
    }
    pub fn proposed(&self) -> &EnvironmentRecord {
        &self.proposed
    }
    pub fn provider_binding(&self) -> Option<&ProviderBindingCommit> {
        self.provider_binding.as_ref()
    }
    pub fn event(&self) -> &EventDraft {
        &self.event
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedEnvironment {
    pub record: EnvironmentRecord,
    pub replayed: bool,
}

/// Transactional persistence seam. Implementations must re-read the current record
/// under the aggregate write lock, enforce exact version/status CAS and immutable
/// identity/config equality, and recheck authoritative Attempt/Invocation/control
/// lease/checkpoint/Effect holds in the same transaction before accepting transitions
/// into CHECKPOINTING or DESTROYING. Caller-supplied domain decisions are not storage
/// proof and must not replace these checks.
/// `create_environment` accepts only a previously authorized PROVISIONING record and
/// atomically commits that record, its initial event, and request receipt. This port
/// does not reserve budget or invoke providers; provider admission must remain blocked
/// until a separate budget reservation is atomically integrated. The idempotency receipt is owner-scoped by
/// `(principal_id, request_id)` and binds the canonical Operator command body/digest.
/// Adapters must recompute and compare the digest. Provider outputs and opaque binding
/// bytes are not part of that body. Its request constructor rejects WORKSPACE_PERSISTENT;
/// only ATTEMPT and TASK_RETAINED are supported until typed preview consumption exists.
/// `transition_environment` atomically commits the
/// versioned record, event, and idempotency receipt. Neither method invokes a provider.
/// A transition to READY must atomically persist the supplied `ProviderBindingCommit`
/// in Runtime-local storage. The adapter revalidates its Environment/Runtime/provider
/// match, active Runtime incarnation, and observed/expiry times in that transaction;
/// it must never put the locator in durable state, events, backups, or projections.
pub trait EnvironmentStore: Send + Sync {
    /// Return at most `request.limit()` records in stable Environment ID order,
    /// scoped to the requested Workspace.
    fn list_environments(
        &self,
        request: EnvironmentListRequest,
    ) -> Result<Vec<EnvironmentRecord>, StoreError>;
    fn get_environment(
        &self,
        workspace_id: &str,
        environment_id: &str,
    ) -> Result<Option<EnvironmentRecord>, StoreError>;
    fn create_environment(
        &self,
        request: EnvironmentCreateRequest,
    ) -> Result<CommittedEnvironment, StoreError>;
    fn transition_environment(
        &self,
        request: EnvironmentLifecycleRequest,
    ) -> Result<CommittedEnvironment, StoreError>;
    /// Atomically changes only a suspended persistent Environment's supported sharing
    /// boundary and owner. Must recheck all represented active-use holds under the
    /// same SQLite write transaction and append the scope event/receipt with the row.
    fn change_environment_sharing_scope(
        &self,
        request: EnvironmentSharingScopeChangeRequest,
    ) -> Result<CommittedEnvironment, StoreError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_environment::{
        EnvironmentLifecycle, LifecycleDecision, ProviderProvisionResult, ProviderReadinessProof,
        new_environment,
    };

    fn id<T>(
        value: &str,
        constructor: impl FnOnce(String) -> Result<T, EnvironmentValidationError>,
    ) -> T {
        constructor(value.to_owned()).unwrap()
    }

    fn identity() -> EnvironmentIdentity {
        EnvironmentIdentity {
            environment_id: id("env-1", EnvironmentId::new),
            runtime_id: id("runtime-1", RuntimeId::new),
            owner_workspace_id: id("workspace-1", WorkspaceId::new),
            created_by_incarnation_id: None,
            owner: EnvironmentOwner {
                task_id: None,
                attempt_id: None,
                coworker_id: None,
                principal_id: None,
            },
        }
    }

    fn config() -> EnvironmentConfig {
        EnvironmentConfig {
            name: "Workspace Environment".to_owned(),
            provider_kind: "local-workspace".to_owned(),
            class: EnvironmentClass::LocalWorkspace,
            lifetime: EnvironmentLifetime::WorkspacePersistent,
            sharing_scope: EnvironmentSharingScope::WorkspaceShared,
            source_resources: vec![PinnedSourceResource {
                resource_id: id("resource-1", ResourceId::new),
                revision_id: id("revision-1", ResourceRevisionId::new),
                content_digest: format!("sha256:{}", "a".repeat(64)),
            }],
            resource_limits: ResourceLimits {
                cpu_millis: 1000,
                memory_bytes: 1024,
                storage_bytes: 4096,
                max_processes: Some(4),
                max_lifetime_seconds: Some(3600),
            },
            network_policy: NetworkPolicy {
                mode: NetworkMode::None,
                allowed_domains: vec![],
                max_response_bytes: 0,
                max_download_bytes: 0,
                deny_private_networks: true,
            },
            budget_ceiling: BudgetCeiling {
                max_wall_time_ms: Some(60000),
                max_cost_minor_units: None,
                currency: None,
                max_tokens: None,
                max_child_attempts: None,
                max_concurrency: Some(1),
            },
            budget_enforcement_policy: BudgetEnforcementPolicy::RequireProviderEnforced,
            budget_enforcement: BudgetEnforcement::ProviderEnforced,
            provision_preview_digest: Some(format!("sha256:{}", "b".repeat(64))),
            retention_expires_at: None,
            backup_policy: EnvironmentBackupPolicy::Excluded,
            isolation: IsolationSpec {
                filesystem: FilesystemIsolation::PrivateCopy,
                process: ProcessIsolation::Container,
                network: NetworkMode::None,
                write_scope: ResourceScope {
                    workspace_id: id("workspace-1", WorkspaceId::new),
                    resource_ids: vec![],
                },
            },
        }
    }

    fn applied(
        decision: Result<LifecycleDecision, domain_environment::EnvironmentError>,
    ) -> EnvironmentRecord {
        match decision.unwrap() {
            LifecycleDecision::Applied(record) => record,
            LifecycleDecision::ReconciliationRequired(_) => panic!("unexpected reconciliation"),
        }
    }

    fn provisioning() -> EnvironmentRecord {
        let record = new_environment(
            identity(),
            config(),
            "2026-10-09T00:00:00Z",
            "2026-10-09T00:00:00Z",
        )
        .unwrap();
        applied(record.request_provision(record.expected_state()))
    }

    fn incarnation() -> RuntimeIncarnationId {
        id("incarnation-1", RuntimeIncarnationId::new)
    }

    fn proof(record: &EnvironmentRecord) -> ProviderReadinessProof {
        ProviderReadinessProof {
            environment_id: record.identity().environment_id.clone(),
            runtime_id: record.identity().runtime_id.clone(),
            runtime_incarnation_id: incarnation(),
            provider_kind: record.config().provider_kind.clone(),
            source_resources: record.config().source_resources.clone(),
            isolation: record.config().isolation.clone(),
            health: EnvironmentHealth::Healthy,
            budget_enforcement: BudgetEnforcement::ProviderEnforced,
        }
    }

    fn ready() -> EnvironmentRecord {
        let pending = provisioning();
        applied(pending.apply_provision_result(
            pending.expected_state(),
            &incarnation(),
            ProviderProvisionResult::Ready(proof(&pending)),
        ))
    }

    fn event_draft() -> EventDraft {
        EventDraft {
            event_id: "event-1".to_owned(),
            workspace_id: "workspace-1".to_owned(),
            entity_type: "Environment".to_owned(),
            entity_id: "env-1".to_owned(),
            origin_runtime_id: "runtime-1".to_owned(),
            entity_revision: 1,
            hlc_timestamp: "2026-10-09T00:00:00Z".to_owned(),
            correlation_id: "request-1".to_owned(),
            causation_id: None,
            schema_version: 1,
            event_type: "environment.created.v1".to_owned(),
            payload: serde_json::json!({"status":"PROVISIONING"}),
            recorded_at: "2026-10-09T00:00:00Z".to_owned(),
        }
    }

    fn request_identity() -> EnvironmentRequestIdentity {
        EnvironmentRequestIdentity::new(
            id("principal-1", PrincipalId::new),
            "request-1",
            br#"{"environment_id":"env-1"}"#.to_vec(),
            format!("sha256:{}", "c".repeat(64)),
        )
        .unwrap()
    }

    #[test]
    fn environment_creation_request_rejects_persistent_without_preview_transaction() {
        let pending = provisioning();
        let error = EnvironmentCreateRequest::new(request_identity(), pending, event_draft())
            .expect_err("persistent creation has no typed preview-consumption contract");
        assert!(error.to_string().contains("typed preview consumption"));

        let mut scoped_identity = identity();
        scoped_identity.owner.task_id = Some(id("task-1", TaskId::new));
        let mut scoped_config = config();
        scoped_config.lifetime = EnvironmentLifetime::TaskRetained;
        scoped_config.sharing_scope = EnvironmentSharingScope::TaskShared;
        scoped_config.provision_preview_digest = None;
        let scoped = new_environment(
            scoped_identity,
            scoped_config,
            "2026-10-09T00:00:00Z",
            "2026-10-09T00:00:00Z",
        )
        .unwrap();
        let scoped = applied(scoped.request_provision(scoped.expected_state()));
        let scoped = scoped.with_updated_at("2026-10-09T00:00:05Z").unwrap();
        let mut mismatched_event = event_draft();
        mismatched_event.recorded_at = "2026-10-09T00:00:06Z".to_owned();
        assert!(
            EnvironmentCreateRequest::new(request_identity(), scoped.clone(), mismatched_event,)
                .is_err()
        );

        let mut matching_event = event_draft();
        matching_event.recorded_at = scoped.updated_at().to_owned();
        let request = EnvironmentCreateRequest::new(request_identity(), scoped, matching_event)
            .expect("task-retained creation is supported by this port");
        assert_eq!(request.identity().principal_id().as_str(), "principal-1");
        assert_eq!(request.request_id(), "request-1");
        assert_eq!(
            request.identity().canonical_request_body(),
            br#"{"environment_id":"env-1"}"#
        );
    }

    #[test]
    fn environment_list_request_requires_explicit_positive_limit() {
        let workspace = id("workspace-1", WorkspaceId::new);
        assert!(EnvironmentListRequest::new(workspace.clone(), None, 0).is_err());
        let request =
            EnvironmentListRequest::new(workspace, Some(id("env-5", EnvironmentId::new)), 25)
                .unwrap();
        assert_eq!(request.limit(), 25);
        assert_eq!(request.after_environment_id().unwrap().as_str(), "env-5");
    }

    #[test]
    fn ready_lifecycle_request_requires_matching_runtime_local_binding() {
        let ready = ready();
        let expected = ready.expected_state();
        assert!(
            EnvironmentLifecycleRequest::new(
                request_identity(),
                ready.identity().environment_id.clone(),
                ready.identity().owner_workspace_id.clone(),
                expected,
                ready.clone(),
                None,
                event_draft(),
            )
            .is_err()
        );

        let binding = ProviderBindingCommit::new(
            ready.identity().environment_id.clone(),
            ready.identity().runtime_id.clone(),
            incarnation(),
            ready.config().provider_kind.clone(),
            "opaque:runtime-local-ref",
            "2026-10-09T00:00:05Z",
            Some("2026-10-10T00:00:00Z".to_owned()),
        )
        .unwrap();
        let stamped_ready = ready.with_updated_at("2026-10-09T00:00:06Z").unwrap();
        assert!(
            EnvironmentLifecycleRequest::new(
                request_identity(),
                stamped_ready.identity().environment_id.clone(),
                stamped_ready.identity().owner_workspace_id.clone(),
                expected,
                stamped_ready.clone(),
                Some(binding.clone()),
                event_draft(),
            )
            .is_err()
        );
        let mut matching_event = event_draft();
        matching_event.recorded_at = stamped_ready.updated_at().to_owned();
        let request = EnvironmentLifecycleRequest::new(
            request_identity(),
            stamped_ready.identity().environment_id.clone(),
            stamped_ready.identity().owner_workspace_id.clone(),
            expected,
            stamped_ready,
            Some(binding),
            matching_event,
        )
        .expect("READY must be paired with a local binding");
        assert!(request.provider_binding().is_some());
    }
}
