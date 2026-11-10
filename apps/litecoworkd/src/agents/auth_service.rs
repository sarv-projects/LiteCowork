//! Runtime-local native sign-in begin and auth observation (specs/002 T07).
//!
//! LiteCowork never captures a credential here. A module declares its own auth
//! methods in its descriptor; `begin` launches only a declared native flow through a
//! launcher boundary and records a bounded `AUTHENTICATING` observation. Auth state
//! comes only from module-reported signals recorded with `record_outcome`; an
//! observation that expires without an outcome reads back as `UNKNOWN`, never as
//! success. Observations are Runtime-local and in-memory: a daemon restart returns
//! every module to `UNKNOWN` until it is observed again (T13).

#![allow(
    dead_code,
    reason = "completion and cancel hooks are exercised by tests until a qualified module declares a native login"
)]

use crate::agents::control_descriptor::{
    AgentAuthMethodDescriptor, AgentAuthMethodKind, AgentAuthState, AgentControlDescriptor,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Longest a native sign-in may stay in flight before it reads back as `UNKNOWN`.
const AUTHENTICATING_WINDOW_SECONDS: i64 = 300;
/// Longest an outcome observation stays valid.
const OUTCOME_WINDOW_SECONDS: i64 = 900;

/// Launches the module's own native login. Implementations must not accept or
/// return any credential bytes.
pub(crate) trait NativeLoginLauncher: Send + Sync {
    fn launch(&self, module_id: &str, auth_method_id: &str) -> Result<(), LaunchError>;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum LaunchError {
    /// No qualified launcher exists for this module on this OS.
    Unsupported,
    Failed,
}

/// Production launcher: V1 ships no module with a qualified native login launch, so
/// every launch is refused rather than guessed.
pub(crate) struct UnqualifiedLauncher;

impl NativeLoginLauncher for UnqualifiedLauncher {
    fn launch(&self, _module_id: &str, _auth_method_id: &str) -> Result<(), LaunchError> {
        Err(LaunchError::Unsupported)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthObservation {
    pub state: AgentAuthState,
    pub auth_method_id: String,
    pub observed_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum BeginOutcome {
    /// A native flow was launched now.
    Started {
        expires_at: i64,
    },
    /// A flow was already in flight; no second launch happened.
    InFlight {
        expires_at: i64,
    },
    SecretRequired,
    NativeConfigRequired,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum BeginError {
    /// The method id is not declared by the module's fresh descriptor.
    MethodNotDeclared,
    /// The declared method kind has no qualified begin path in V1.
    Unsupported,
    LaunchFailed,
}

pub(crate) struct AgentAuthService {
    launcher: Arc<dyn NativeLoginLauncher>,
    observations: Mutex<HashMap<String, AuthObservation>>,
}

impl AgentAuthService {
    pub(crate) fn new(launcher: Arc<dyn NativeLoginLauncher>) -> Self {
        Self {
            launcher,
            observations: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn shared() -> Arc<Self> {
        static SERVICE: OnceLock<Arc<AgentAuthService>> = OnceLock::new();
        SERVICE
            .get_or_init(|| Arc::new(Self::new(Arc::new(UnqualifiedLauncher))))
            .clone()
    }

    /// Begins one declared method. The observation map lock is held across the
    /// launch so two concurrent begins can never both spawn a native flow.
    pub(crate) fn begin(
        &self,
        descriptor: &AgentControlDescriptor,
        auth_method_id: &str,
        now_epoch: i64,
    ) -> Result<BeginOutcome, BeginError> {
        let method: &AgentAuthMethodDescriptor = descriptor
            .auth
            .methods
            .iter()
            .find(|method| method.auth_method_id == auth_method_id)
            .ok_or(BeginError::MethodNotDeclared)?;
        match method.kind {
            AgentAuthMethodKind::SecretSlot => return Ok(BeginOutcome::SecretRequired),
            AgentAuthMethodKind::NativeConfigOnly => {
                return Ok(BeginOutcome::NativeConfigRequired);
            }
            AgentAuthMethodKind::ExternalHandoff | AgentAuthMethodKind::DeviceCode => {
                return Err(BeginError::Unsupported);
            }
            AgentAuthMethodKind::NativeFlow => {}
        }
        let mut observations = self
            .observations
            .lock()
            .map_err(|_| BeginError::LaunchFailed)?;
        if let Some(existing) = observations.get(&descriptor.module_id)
            && existing.state == AgentAuthState::Authenticating
            && existing.expires_at > now_epoch
        {
            return Ok(BeginOutcome::InFlight {
                expires_at: existing.expires_at,
            });
        }
        match self.launcher.launch(&descriptor.module_id, auth_method_id) {
            Ok(()) => {
                let expires_at = now_epoch + AUTHENTICATING_WINDOW_SECONDS;
                observations.insert(
                    descriptor.module_id.clone(),
                    AuthObservation {
                        state: AgentAuthState::Authenticating,
                        auth_method_id: auth_method_id.to_owned(),
                        observed_at: now_epoch,
                        expires_at,
                    },
                );
                Ok(BeginOutcome::Started { expires_at })
            }
            Err(LaunchError::Unsupported) => Err(BeginError::Unsupported),
            Err(LaunchError::Failed) => {
                observations.insert(
                    descriptor.module_id.clone(),
                    AuthObservation {
                        state: AgentAuthState::Error,
                        auth_method_id: auth_method_id.to_owned(),
                        observed_at: now_epoch,
                        expires_at: now_epoch + OUTCOME_WINDOW_SECONDS,
                    },
                );
                Err(BeginError::LaunchFailed)
            }
        }
    }

    /// Records a module-reported outcome after a bounded re-probe. Only the closed
    /// non-`AUTHENTICATING` states are accepted. No binding is touched.
    pub(crate) fn record_outcome(
        &self,
        module_id: &str,
        auth_method_id: &str,
        state: AgentAuthState,
        now_epoch: i64,
    ) {
        if state == AgentAuthState::Authenticating {
            return;
        }
        if let Ok(mut observations) = self.observations.lock() {
            observations.insert(
                module_id.to_owned(),
                AuthObservation {
                    state,
                    auth_method_id: auth_method_id.to_owned(),
                    observed_at: now_epoch,
                    expires_at: now_epoch + OUTCOME_WINDOW_SECONDS,
                },
            );
        }
    }

    /// A cancelled sign-in is a failed observation, never an authenticated one.
    pub(crate) fn cancel(&self, module_id: &str, now_epoch: i64) {
        let method = self
            .observations
            .lock()
            .ok()
            .and_then(|map| map.get(module_id).map(|o| o.auth_method_id.clone()))
            .unwrap_or_default();
        self.record_outcome(module_id, &method, AgentAuthState::NeedsAuth, now_epoch);
    }

    /// The latest unexpired state. An expired observation (including an
    /// `AUTHENTICATING` window that never reported) reads back as `UNKNOWN`.
    pub(crate) fn state(&self, module_id: &str, now_epoch: i64) -> AgentAuthState {
        self.observations
            .lock()
            .ok()
            .and_then(|map| map.get(module_id).cloned())
            .filter(|observation| observation.expires_at > now_epoch)
            .map_or(AgentAuthState::Unknown, |observation| observation.state)
    }

    /// Applies the observed state to a freshly built descriptor and reseals it.
    pub(crate) fn overlay(
        &self,
        descriptor: &mut AgentControlDescriptor,
        now_epoch: i64,
    ) -> Result<(), ()> {
        let state = self.state(&descriptor.module_id, now_epoch);
        if descriptor.auth.state == state {
            return Ok(());
        }
        descriptor.auth.state = state;
        descriptor.descriptor_digest = descriptor.computed_digest().map_err(|_| ())?;
        descriptor.validate().map_err(|_| ())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::agents::modules::opencode;
    use crate::agents::opencode_profile_probe::{OpenCodeProbeReadiness, OpenCodeProfileProbe};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingLauncher(AtomicUsize, Result<(), LaunchError>);

    impl NativeLoginLauncher for CountingLauncher {
        fn launch(&self, _module: &str, _method: &str) -> Result<(), LaunchError> {
            self.0.fetch_add(1, Ordering::SeqCst);
            match &self.1 {
                Ok(()) => Ok(()),
                Err(LaunchError::Unsupported) => Err(LaunchError::Unsupported),
                Err(LaunchError::Failed) => Err(LaunchError::Failed),
            }
        }
    }

    fn descriptor(kind: Option<AgentAuthMethodKind>) -> AgentControlDescriptor {
        let mut probe = OpenCodeProfileProbe::initial();
        probe.protocol_version = Some("2.0.32".to_owned());
        probe.probe_readiness = OpenCodeProbeReadiness::Complete;
        probe.host_process_stopped = true;
        let mut descriptor = opencode::build_descriptor(
            &probe,
            "runtime",
            "incarnation",
            time::OffsetDateTime::from_unix_timestamp(1_800_000_000).expect("time"),
            900,
        )
        .expect("descriptor builds");
        if let Some(kind) = kind {
            descriptor.auth.methods.push(AgentAuthMethodDescriptor {
                auth_method_id: "native-login".to_owned(),
                display_name: "Sign in".to_owned(),
                kind,
                credential_slot_id: None,
                requires_restart: false,
            });
        }
        descriptor
    }

    fn service(result: Result<(), LaunchError>) -> (AgentAuthService, Arc<CountingLauncher>) {
        let launcher = Arc::new(CountingLauncher(AtomicUsize::new(0), result));
        (AgentAuthService::new(launcher.clone()), launcher)
    }

    #[test]
    fn agent_auth_undeclared_method_is_refused_without_a_launch() {
        let (service, launcher) = service(Ok(()));
        let descriptor = descriptor(None);
        assert_eq!(
            service.begin(&descriptor, "native-login", 100),
            Err(BeginError::MethodNotDeclared)
        );
        assert_eq!(launcher.0.load(Ordering::SeqCst), 0);
        assert_eq!(
            service.state(&descriptor.module_id, 100),
            AgentAuthState::Unknown
        );
    }

    #[test]
    fn agent_auth_declared_native_flow_authenticates_once_and_returns_in_flight() {
        let (service, launcher) = service(Ok(()));
        let descriptor = descriptor(Some(AgentAuthMethodKind::NativeFlow));
        assert_eq!(
            service.begin(&descriptor, "native-login", 100),
            Ok(BeginOutcome::Started { expires_at: 400 })
        );
        assert_eq!(
            service.begin(&descriptor, "native-login", 150),
            Ok(BeginOutcome::InFlight { expires_at: 400 })
        );
        assert_eq!(launcher.0.load(Ordering::SeqCst), 1);
        assert_eq!(
            service.state(&descriptor.module_id, 150),
            AgentAuthState::Authenticating
        );
    }

    #[test]
    fn agent_auth_unreported_flow_expires_to_unknown_never_authenticated() {
        let (service, _) = service(Ok(()));
        let descriptor = descriptor(Some(AgentAuthMethodKind::NativeFlow));
        service
            .begin(&descriptor, "native-login", 100)
            .expect("begin");
        assert_eq!(
            service.state(&descriptor.module_id, 400),
            AgentAuthState::Unknown
        );
    }

    #[test]
    fn agent_auth_cancel_and_failure_record_closed_states() {
        let (service, _) = service(Ok(()));
        let descriptor = descriptor(Some(AgentAuthMethodKind::NativeFlow));
        service
            .begin(&descriptor, "native-login", 100)
            .expect("begin");
        service.cancel(&descriptor.module_id, 120);
        assert_eq!(
            service.state(&descriptor.module_id, 121),
            AgentAuthState::NeedsAuth
        );
        let (failing, _) = service_failing();
        assert_eq!(
            failing.begin(&descriptor, "native-login", 100),
            Err(BeginError::LaunchFailed)
        );
        assert_eq!(
            failing.state(&descriptor.module_id, 101),
            AgentAuthState::Error
        );
    }

    fn service_failing() -> (AgentAuthService, Arc<CountingLauncher>) {
        service(Err(LaunchError::Failed))
    }

    #[test]
    fn agent_auth_unqualified_launcher_refuses_and_records_nothing() {
        let service = AgentAuthService::new(Arc::new(UnqualifiedLauncher));
        let descriptor = descriptor(Some(AgentAuthMethodKind::NativeFlow));
        assert_eq!(
            service.begin(&descriptor, "native-login", 100),
            Err(BeginError::Unsupported)
        );
        assert_eq!(
            service.state(&descriptor.module_id, 100),
            AgentAuthState::Unknown
        );
    }

    #[test]
    fn agent_auth_non_native_kinds_never_launch() {
        let (service, launcher) = service(Ok(()));
        assert_eq!(
            service.begin(
                &descriptor(Some(AgentAuthMethodKind::SecretSlot)),
                "native-login",
                1
            ),
            Ok(BeginOutcome::SecretRequired)
        );
        assert_eq!(
            service.begin(
                &descriptor(Some(AgentAuthMethodKind::DeviceCode)),
                "native-login",
                1
            ),
            Err(BeginError::Unsupported)
        );
        assert_eq!(launcher.0.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn agent_auth_overlay_reseals_the_descriptor_digest() {
        let (service, _) = service(Ok(()));
        let mut descriptor = descriptor(None);
        let before = descriptor.descriptor_digest.clone();
        service.record_outcome(
            &descriptor.module_id,
            "native-login",
            AgentAuthState::Authenticated,
            100,
        );
        service.overlay(&mut descriptor, 101).expect("overlay");
        assert_eq!(descriptor.auth.state, AgentAuthState::Authenticated);
        assert_ne!(descriptor.descriptor_digest, before);
        descriptor
            .validate()
            .expect("resealed descriptor validates");
    }
}
