//! Keep-awake power assertion state machine (011-T12), daemon side.
//!
//! `os_state` is HELD only after the OS backend confirmed the assertion; a refused or failed
//! acquire is UNKNOWN, never HELD, and a mode that needs a power-source reading the OS cannot
//! give is UNSUPPORTED and cannot be selected. Drain releases the assertion before STOPPING.
//!
//! Not wired into the daemon yet: the "work runs" input must come from
//! `RuntimeLifecycleStore::read_runtime_stop_observation` (admitted turns and Attempts) and
//! `RuntimeLifecycleView.power_assertion` must be published, both of which are still open
//! 011-T12 steps. The Windows `PowerCreateRequest` backend needs a native Windows runner and
//! is not implemented. The Linux backend holds a `systemd-inhibit` child process and could not
//! be exercised here because this container has no logind.

use std::process::{Child, Command, Stdio};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PowerMode {
    Off,
    WhileWorkRuns,
    WhileWorkRunsOnPower,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OsState {
    Held,
    Released,
    Unsupported,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PowerBackendError(pub(crate) String);

pub(crate) trait PowerBackend {
    /// False when this OS has no usable provider at all.
    fn available(&self) -> bool;
    /// Takes the assertion. `Ok` means the OS confirmed it.
    fn acquire(&mut self) -> Result<(), PowerBackendError>;
    fn release(&mut self) -> Result<(), PowerBackendError>;
    /// `Some(true)` on battery, `Some(false)` on mains, `None` when the OS cannot report it.
    fn on_battery(&self) -> Option<bool>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PowerAssertionSnapshot {
    pub(crate) mode: PowerMode,
    pub(crate) os_state: OsState,
    pub(crate) observed_at: String,
}

pub(crate) struct PowerAssertion<B: PowerBackend> {
    backend: B,
    mode: PowerMode,
    os_state: OsState,
    observed_at: String,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum PowerModeError {
    Unsupported,
}

impl<B: PowerBackend> PowerAssertion<B> {
    pub(crate) fn new(backend: B, observed_at: &str) -> Self {
        let os_state = if backend.available() {
            OsState::Released
        } else {
            OsState::Unsupported
        };
        Self {
            backend,
            mode: PowerMode::Off,
            os_state,
            observed_at: observed_at.to_owned(),
        }
    }

    /// Selecting a mode never reports HELD by itself; only `reconcile` can after an OS success.
    pub(crate) fn set_mode(&mut self, mode: PowerMode) -> Result<(), PowerModeError> {
        if mode != PowerMode::Off && !self.backend.available() {
            return Err(PowerModeError::Unsupported);
        }
        if mode == PowerMode::WhileWorkRunsOnPower && self.backend.on_battery().is_none() {
            return Err(PowerModeError::Unsupported);
        }
        self.mode = mode;
        Ok(())
    }

    fn desired(&self, work_running: bool) -> bool {
        match self.mode {
            PowerMode::Off => false,
            PowerMode::WhileWorkRuns => work_running,
            PowerMode::WhileWorkRunsOnPower => {
                work_running && self.backend.on_battery() == Some(false)
            }
        }
    }

    pub(crate) fn reconcile(&mut self, work_running: bool, now: &str) -> PowerAssertionSnapshot {
        if self.os_state != OsState::Unsupported {
            let desired = self.desired(work_running);
            let held = self.os_state == OsState::Held;
            if desired && !held {
                self.os_state = match self.backend.acquire() {
                    Ok(()) => OsState::Held,
                    Err(_) => OsState::Unknown,
                };
            } else if !desired && (held || self.os_state == OsState::Unknown) {
                self.os_state = match self.backend.release() {
                    Ok(()) => OsState::Released,
                    Err(_) => OsState::Unknown,
                };
            }
        }
        self.observed_at = now.to_owned();
        self.snapshot()
    }

    /// Called when the Runtime starts DRAINING, before STOPPING.
    pub(crate) fn drain(&mut self, now: &str) -> PowerAssertionSnapshot {
        self.mode = PowerMode::Off;
        self.reconcile(false, now)
    }

    pub(crate) fn snapshot(&self) -> PowerAssertionSnapshot {
        PowerAssertionSnapshot {
            mode: self.mode,
            os_state: self.os_state,
            observed_at: self.observed_at.clone(),
        }
    }
}

/// Linux provider: a held `systemd-inhibit` child. The assertion exists while the child lives.
#[derive(Default)]
pub(crate) struct SystemdInhibitBackend {
    child: Option<Child>,
}

impl SystemdInhibitBackend {
    fn mains_online() -> Option<bool> {
        let entries = std::fs::read_dir("/sys/class/power_supply").ok()?;
        let mut saw_mains = false;
        for entry in entries.flatten() {
            let path = entry.path();
            if std::fs::read_to_string(path.join("type")).ok()?.trim() != "Mains" {
                continue;
            }
            saw_mains = true;
            if std::fs::read_to_string(path.join("online")).ok()?.trim() == "1" {
                return Some(false);
            }
        }
        saw_mains.then_some(true)
    }
}

impl PowerBackend for SystemdInhibitBackend {
    fn available(&self) -> bool {
        std::env::var_os("PATH").is_some_and(|paths| {
            std::env::split_paths(&paths).any(|dir| dir.join("systemd-inhibit").is_file())
        })
    }

    fn acquire(&mut self) -> Result<(), PowerBackendError> {
        if self.child.is_some() {
            return Ok(());
        }
        let mut child = Command::new("systemd-inhibit")
            .args([
                "--what=sleep:idle",
                "--who=LiteCowork",
                "--why=Work is running",
                "--mode=block",
                "sleep",
                "infinity",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| PowerBackendError(error.to_string()))?;
        std::thread::sleep(std::time::Duration::from_millis(300));
        match child.try_wait() {
            Ok(None) => {
                self.child = Some(child);
                Ok(())
            }
            Ok(Some(status)) => Err(PowerBackendError(format!("inhibitor exited: {status}"))),
            Err(error) => {
                let _ = child.kill();
                Err(PowerBackendError(error.to_string()))
            }
        }
    }

    fn release(&mut self) -> Result<(), PowerBackendError> {
        if let Some(mut child) = self.child.take() {
            child
                .kill()
                .map_err(|error| PowerBackendError(error.to_string()))?;
            let _ = child.wait();
        }
        Ok(())
    }

    fn on_battery(&self) -> Option<bool> {
        Self::mains_online().map(|online| !online)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeBackend {
        available: bool,
        battery: Option<bool>,
        fail_acquire: bool,
        fail_release: bool,
        acquires: u32,
        releases: u32,
    }

    impl FakeBackend {
        fn new() -> Self {
            Self {
                available: true,
                battery: Some(false),
                fail_acquire: false,
                fail_release: false,
                acquires: 0,
                releases: 0,
            }
        }
    }

    impl PowerBackend for FakeBackend {
        fn available(&self) -> bool {
            self.available
        }
        fn acquire(&mut self) -> Result<(), PowerBackendError> {
            self.acquires += 1;
            if self.fail_acquire {
                Err(PowerBackendError("refused".to_owned()))
            } else {
                Ok(())
            }
        }
        fn release(&mut self) -> Result<(), PowerBackendError> {
            self.releases += 1;
            if self.fail_release {
                Err(PowerBackendError("stuck".to_owned()))
            } else {
                Ok(())
            }
        }
        fn on_battery(&self) -> Option<bool> {
            self.battery
        }
    }

    const T0: &str = "2026-11-10T00:00:00Z";
    const T1: &str = "2026-11-10T00:00:01Z";

    #[test]
    fn selecting_a_mode_alone_never_reports_held() {
        let mut assertion = PowerAssertion::new(FakeBackend::new(), T0);
        assertion
            .set_mode(PowerMode::WhileWorkRuns)
            .expect("supported");
        assert_eq!(assertion.snapshot().os_state, OsState::Released);
        assert_eq!(assertion.backend.acquires, 0);
    }

    #[test]
    fn held_appears_only_after_the_os_call_succeeds() {
        let mut assertion = PowerAssertion::new(FakeBackend::new(), T0);
        assertion
            .set_mode(PowerMode::WhileWorkRuns)
            .expect("supported");
        assert_eq!(assertion.reconcile(true, T1).os_state, OsState::Held);
        assert_eq!(assertion.reconcile(true, T1).os_state, OsState::Held);
        assert_eq!(
            assertion.backend.acquires, 1,
            "no repeated acquire while held"
        );
        assert_eq!(assertion.reconcile(false, T1).os_state, OsState::Released);
        assert_eq!(assertion.backend.releases, 1);
    }

    #[test]
    fn an_os_refusal_is_unknown_never_held() {
        let mut backend = FakeBackend::new();
        backend.fail_acquire = true;
        let mut assertion = PowerAssertion::new(backend, T0);
        assertion
            .set_mode(PowerMode::WhileWorkRuns)
            .expect("supported");
        assert_eq!(assertion.reconcile(true, T1).os_state, OsState::Unknown);
    }

    #[test]
    fn a_failed_release_is_unknown_and_drain_still_clears_the_mode() {
        let mut backend = FakeBackend::new();
        backend.fail_release = true;
        let mut assertion = PowerAssertion::new(backend, T0);
        assertion
            .set_mode(PowerMode::WhileWorkRuns)
            .expect("supported");
        assertion.reconcile(true, T1);
        let drained = assertion.drain(T1);
        assert_eq!(drained.mode, PowerMode::Off);
        assert_eq!(drained.os_state, OsState::Unknown);
    }

    #[test]
    fn drain_releases_before_stopping() {
        let mut assertion = PowerAssertion::new(FakeBackend::new(), T0);
        assertion
            .set_mode(PowerMode::WhileWorkRuns)
            .expect("supported");
        assertion.reconcile(true, T1);
        let drained = assertion.drain(T1);
        assert_eq!(drained.os_state, OsState::Released);
        assert_eq!(drained.mode, PowerMode::Off);
    }

    #[test]
    fn on_power_mode_is_unsupported_when_the_os_cannot_report_the_power_source() {
        let mut backend = FakeBackend::new();
        backend.battery = None;
        let mut assertion = PowerAssertion::new(backend, T0);
        assert_eq!(
            assertion.set_mode(PowerMode::WhileWorkRunsOnPower),
            Err(PowerModeError::Unsupported)
        );
        assert_eq!(assertion.snapshot().mode, PowerMode::Off);
    }

    #[test]
    fn on_power_mode_does_not_hold_on_battery() {
        let mut backend = FakeBackend::new();
        backend.battery = Some(true);
        let mut assertion = PowerAssertion::new(backend, T0);
        assertion
            .set_mode(PowerMode::WhileWorkRunsOnPower)
            .expect("supported");
        assert_eq!(assertion.reconcile(true, T1).os_state, OsState::Released);
        assert_eq!(assertion.backend.acquires, 0);
    }

    #[test]
    fn an_unavailable_backend_reports_unsupported_and_cannot_be_selected() {
        let mut backend = FakeBackend::new();
        backend.available = false;
        let mut assertion = PowerAssertion::new(backend, T0);
        assert_eq!(assertion.snapshot().os_state, OsState::Unsupported);
        assert_eq!(
            assertion.set_mode(PowerMode::WhileWorkRuns),
            Err(PowerModeError::Unsupported)
        );
        assert_eq!(assertion.reconcile(true, T1).os_state, OsState::Unsupported);
        assert_eq!(assertion.backend.acquires, 0);
    }

    #[test]
    fn power_assertion_backend_on_this_host_never_claims_held_without_an_inhibitor() {
        // Without logind the real backend must refuse; with it, it must hold. Either way the
        // state machine only reports what the OS call returned.
        let mut assertion = PowerAssertion::new(SystemdInhibitBackend::default(), T0);
        if assertion.set_mode(PowerMode::WhileWorkRuns).is_err() {
            assert_eq!(assertion.snapshot().os_state, OsState::Unsupported);
            return;
        }
        let state = assertion.reconcile(true, T1).os_state;
        assert!(
            matches!(state, OsState::Held | OsState::Unknown),
            "{state:?}"
        );
        assert_eq!(assertion.drain(T1).os_state, OsState::Released);
    }
}
