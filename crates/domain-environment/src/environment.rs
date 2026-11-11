//! Canonical Environment vocabulary: identity newtypes, lifecycle/configuration
//! vocabularies, records, and state projections.
//!
//! These types are the shared kernel between the domain lifecycle decisions in this
//! crate and the transactional `EnvironmentStore` port, which stays with the storage
//! adapters in `storage-core`. Validation failures on this vocabulary surface as
//! `EnvironmentValidationError`; storage adapters convert them into their own error
//! taxonomy (`StoreError::Invalid`).

use serde::{Deserialize, Serialize};

/// Observed enforcement of one isolation dimension.
///
/// A positive value means the provider demonstrated the restriction and the Runtime
/// observed it. `Unknown` never satisfies an admission requirement.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnforcementStatus {
    Enforced,
    NotEnforced,
    Unknown,
}

impl EnforcementStatus {
    /// Only a positive observation may satisfy an admission requirement.
    pub fn is_enforced(self) -> bool {
        matches!(self, Self::Enforced)
    }
}

/// Validation failure of a canonical Environment vocabulary value. The message is the
/// exact durable failure description; storage adapters map it onto their own invalid-
/// input error without rewriting it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnvironmentValidationError(String);

impl EnvironmentValidationError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    pub fn message(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for EnvironmentValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for EnvironmentValidationError {}

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, EnvironmentValidationError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(EnvironmentValidationError::new(concat!(
                        stringify!($name),
                        " must be non-empty"
                    )));
                }
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

id_type!(EnvironmentId);
id_type!(RuntimeId);
id_type!(RuntimeIncarnationId);
id_type!(WorkspaceId);
id_type!(TaskId);
id_type!(AttemptId);
id_type!(CoworkerId);
id_type!(PrincipalId);
id_type!(ResourceId);
id_type!(ResourceRevisionId);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnvironmentStatus {
    New,
    Provisioning,
    Ready,
    Busy,
    Checkpointing,
    Suspended,
    Failed,
    Destroying,
    Destroyed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnvironmentHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnvironmentClass {
    LocalWorkspace,
    GitWorktree,
    Container,
    Vm,
    CloudSandbox,
    RemoteMachine,
    Browser,
    Desktop,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnvironmentLifetime {
    Attempt,
    TaskRetained,
    WorkspacePersistent,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnvironmentSharingScope {
    AttemptPrivate,
    TaskShared,
    CoworkerPrivate,
    WorkspaceShared,
    UserShared,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FilesystemIsolation {
    SharedReadonly,
    PrivateCopy,
    Worktree,
    ContainerFs,
    VmFs,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProcessIsolation {
    Host,
    Namespace,
    Container,
    Vm,
    Remote,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NetworkMode {
    None,
    Restricted,
    Default,
    Custom,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceScope {
    pub workspace_id: WorkspaceId,
    pub resource_ids: Vec<ResourceId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IsolationSpec {
    pub filesystem: FilesystemIsolation,
    pub process: ProcessIsolation,
    pub network: NetworkMode,
    pub write_scope: ResourceScope,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceLimits {
    pub cpu_millis: u32,
    pub memory_bytes: u64,
    pub storage_bytes: u64,
    pub max_processes: Option<u32>,
    pub max_lifetime_seconds: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkPolicy {
    pub mode: NetworkMode,
    pub allowed_domains: Vec<String>,
    pub max_response_bytes: u64,
    pub max_download_bytes: u64,
    pub deny_private_networks: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetCeiling {
    pub max_wall_time_ms: Option<u64>,
    pub max_cost_minor_units: Option<u64>,
    pub currency: Option<String>,
    pub max_tokens: Option<u64>,
    pub max_child_attempts: Option<u32>,
    pub max_concurrency: Option<u32>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BudgetEnforcementPolicy {
    RequireProviderEnforced,
    AllowHostMonitored,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BudgetEnforcement {
    ProviderEnforced,
    HostMonitored,
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PinnedSourceResource {
    pub resource_id: ResourceId,
    pub revision_id: ResourceRevisionId,
    pub content_digest: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentOwner {
    pub task_id: Option<TaskId>,
    pub attempt_id: Option<AttemptId>,
    pub coworker_id: Option<CoworkerId>,
    pub principal_id: Option<PrincipalId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentIdentity {
    pub environment_id: EnvironmentId,
    pub runtime_id: RuntimeId,
    pub owner_workspace_id: WorkspaceId,
    pub owner: EnvironmentOwner,
    /// Incarnation that created the Environment, when known. This is immutable and
    /// is not the incarnation currently operating the Environment.
    pub created_by_incarnation_id: Option<RuntimeIncarnationId>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnvironmentBackupPolicy {
    Excluded,
    IncludeCheckpoints,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentConfig {
    pub name: String,
    pub provider_kind: String,
    pub class: EnvironmentClass,
    pub lifetime: EnvironmentLifetime,
    pub sharing_scope: EnvironmentSharingScope,
    pub source_resources: Vec<PinnedSourceResource>,
    pub resource_limits: ResourceLimits,
    pub network_policy: NetworkPolicy,
    pub budget_ceiling: BudgetCeiling,
    pub budget_enforcement_policy: BudgetEnforcementPolicy,
    pub budget_enforcement: BudgetEnforcement,
    /// Set only when a persistent Environment was created by consuming a validated
    /// provision preview. V1 storage requests cannot create that lifetime yet.
    pub provision_preview_digest: Option<String>,
    pub retention_expires_at: Option<String>,
    pub backup_policy: EnvironmentBackupPolicy,
    pub isolation: IsolationSpec,
}

/// Immutable provision identity/config plus lifecycle projection. Construction and
/// persisted restoration go through `domain_environment` validation; storage adapters
/// should use `from_persisted_parts` and reject an error as corrupt state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EnvironmentRecord {
    identity: EnvironmentIdentity,
    config: EnvironmentConfig,
    status: EnvironmentStatus,
    health: EnvironmentHealth,
    created_at: String,
    updated_at: String,
    version: u64,
}

impl EnvironmentRecord {
    pub fn from_persisted_parts(
        identity: EnvironmentIdentity,
        config: EnvironmentConfig,
        status: EnvironmentStatus,
        health: EnvironmentHealth,
        created_at: String,
        updated_at: String,
        version: u64,
    ) -> Result<Self, EnvironmentValidationError> {
        if version == 0 {
            return Err(EnvironmentValidationError::new(
                "Environment version must be positive",
            ));
        }
        Ok(Self {
            identity,
            config,
            status,
            health,
            created_at,
            updated_at,
            version,
        })
    }

    pub fn identity(&self) -> &EnvironmentIdentity {
        &self.identity
    }
    pub fn config(&self) -> &EnvironmentConfig {
        &self.config
    }
    pub const fn status(&self) -> EnvironmentStatus {
        self.status
    }
    pub const fn health(&self) -> EnvironmentHealth {
        self.health
    }
    pub fn created_at(&self) -> &str {
        &self.created_at
    }
    pub fn updated_at(&self) -> &str {
        &self.updated_at
    }

    /// Returns a copy stamped with the caller's transaction time. Lifecycle policy
    /// remains clock-free; persistence request construction requires this value to
    /// match the event's `recorded_at` exactly.
    pub fn with_updated_at(
        &self,
        updated_at: impl Into<String>,
    ) -> Result<Self, EnvironmentValidationError> {
        let updated_at = updated_at.into();
        if updated_at.trim().is_empty() {
            return Err(EnvironmentValidationError::new(
                "Environment updated_at must be non-empty",
            ));
        }
        Ok(Self {
            identity: self.identity.clone(),
            config: self.config.clone(),
            status: self.status,
            health: self.health,
            created_at: self.created_at.clone(),
            updated_at,
            version: self.version,
        })
    }
    pub const fn version(&self) -> u64 {
        self.version
    }

    /// Only the domain lifecycle module may use this copy constructor to propose a
    /// changed state. Store adapters still recheck immutable identity/config on CAS.
    pub fn with_domain_state(
        &self,
        status: EnvironmentStatus,
        health: EnvironmentHealth,
        version: u64,
    ) -> Result<Self, EnvironmentValidationError> {
        if version != self.version.saturating_add(1) {
            return Err(EnvironmentValidationError::new(
                "Environment transition must advance exactly one version",
            ));
        }
        Ok(Self {
            identity: self.identity.clone(),
            config: self.config.clone(),
            status,
            health,
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
            version,
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedState {
    pub version: u64,
    pub status: EnvironmentStatus,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleHolds {
    pub active_attempts: u32,
    pub unsettled_invocations: u32,
    pub unsettled_control_leases: u32,
    pub checkpoint_holds: u32,
    pub unresolved_effects: u32,
}
