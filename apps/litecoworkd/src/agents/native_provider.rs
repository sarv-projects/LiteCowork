//! Closed native provider identity and the OpenCode offer-admission predicate.
//!
//! The stored `AgentProfile` provider key is the single identity that both the
//! probe (which publishes the Runtime offer and registers the endpoint locator)
//! and the Conversation supervisor (which dispatches the native host) must agree
//! on. Parsing it once at the boundary here replaces the fragile string match in
//! `supervisor::execute_agent_turn` that treated the canonical upper-case stored
//! form (`OPENCODE`) and the lower-case literal (`opencode`) as different agents,
//! so a stored OpenCode binding could never be dispatched.
//!
//! Offer compatibility is computed from an observed probe, never hard-coded. Only
//! the exact accepted route (OpenCode CLI 2.0.32, a complete probe and an observed
//! host stop) is published as a compatible `STARTABLE` offer. Every other outcome
//! stays `compatible = false` with a reason, so the unchanged storage admission
//! gate rejects it without a weaker filter.

use super::OpenCodeServerProtocol;
use super::opencode_profile_probe::{OpenCodeProbeReadiness, OpenCodeProfileProbe};

/// Closed set of native providers the daemon can dispatch. Parsed once from the
/// stored profile provider key; an unknown key yields `None` and fails closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativeProviderKey {
    OpenCode,
    ClaudeCode,
    Codex,
}

impl NativeProviderKey {
    /// The canonical stored form is upper case (`OPENCODE`, `CLAUDE_CODE`,
    /// `CODEX`). Parsing is case-insensitive so a legacy lower-case literal
    /// still resolves to the same provider instead of silently falling into the
    /// unsupported arm. Any other value returns `None` and must fail closed.
    pub(crate) fn from_profile_key(stored: &str) -> Option<Self> {
        match stored.trim().to_ascii_uppercase().as_str() {
            "OPENCODE" => Some(Self::OpenCode),
            "CLAUDE_CODE" => Some(Self::ClaudeCode),
            "CODEX" => Some(Self::Codex),
            _ => None,
        }
    }
}

/// Computed Runtime-offer admission for an OpenCode profile probe. Never
/// hard-coded: `compatible` is true only for the exact accepted route.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OfferAdmission {
    pub compatible: bool,
    pub readiness: &'static str,
    pub reason: Option<&'static str>,
}

/// Publishes a truthful compatible offer only for the exact accepted route:
/// OpenCode CLI 2.0.32 (`OpenCodeServerProtocol::V2_0_32`), a complete probe and
/// an observed host stop. 2.0.26 and 2.0.28 stay observation-only. Every negative
/// keeps `compatible = false` and names the reason in the offer constraints, so
/// the unchanged storage gate (`has_fresh_compatible_endpoint`,
/// `resolve_conversation_session_host_in_transaction`) rejects the binding.
pub(crate) fn opencode_offer_admission(probe: &OpenCodeProfileProbe) -> OfferAdmission {
    let version_is_exact = probe
        .protocol_version
        .as_deref()
        .and_then(OpenCodeServerProtocol::for_cli_version)
        == Some(OpenCodeServerProtocol::V2_0_32);
    if !version_is_exact {
        return OfferAdmission {
            compatible: false,
            readiness: "UNAVAILABLE",
            reason: Some("OpenCode CLI version is not the accepted 2.0.32 route"),
        };
    }
    if probe.probe_readiness != OpenCodeProbeReadiness::Complete {
        return OfferAdmission {
            compatible: false,
            readiness: "DEGRADED",
            reason: Some("OpenCode profile probe did not complete"),
        };
    }
    if !probe.host_process_stopped {
        return OfferAdmission {
            compatible: false,
            readiness: "DEGRADED",
            reason: Some("OpenCode host process stop was not observed"),
        };
    }
    OfferAdmission {
        compatible: true,
        readiness: "STARTABLE",
        reason: None,
    }
}

/// Whether the probe admits a bindable Conversation offer. This is the single
/// predicate behind both the Runtime offer flag and
/// `OpenCodeProfileProbe::profile_admission_supported` (Conversation admission
/// only; Task admission stays unsupported and keeps its own copy flag).
pub(crate) fn opencode_profile_admission_supported(probe: &OpenCodeProfileProbe) -> bool {
    opencode_offer_admission(probe).compatible
}

/// Shared OpenCode executable resolver used by both the probe and the supervisor
/// so the probed and dispatched executables are the same canonical path.
///
/// It applies the Windows npm/bun shim rules (`.exe`, `.cmd`, `%APPDATA%\npm`,
/// `.bun\bin`) and canonicalizes the result. `resolve_local_executable("opencode")`
/// previously joined the bare name onto each `PATH` entry, so a Windows
/// `opencode.cmd`/`opencode.exe` was never found and the probe returned
/// `404 AGENT_UNAVAILABLE`, while the supervisor resolved a different executable.
pub(crate) fn resolve_opencode_executable() -> Option<std::path::PathBuf> {
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();

    #[cfg(windows)]
    {
        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            let bun_bin = std::path::PathBuf::from(&user_profile)
                .join(".bun")
                .join("bin");
            candidates.push(bun_bin.join("opencode.exe"));
            candidates.push(bun_bin.join("opencode.cmd"));
        }
        if let Some(app_data) = std::env::var_os("APPDATA") {
            let npm = std::path::PathBuf::from(&app_data).join("npm");
            candidates.push(npm.join("opencode.cmd"));
            candidates.push(npm.join("opencode.exe"));
        }
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            let base = std::path::PathBuf::from(&local_app_data);
            candidates.push(base.join("pi-node").join("current").join("opencode.cmd"));
            candidates.push(base.join("pi-node").join("current").join("opencode.exe"));
            candidates.push(base.join("Programs").join("opencode").join("opencode.exe"));
        }
    }

    if let Some(search_path) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(&search_path) {
            #[cfg(windows)]
            {
                candidates.push(directory.join("opencode.exe"));
                candidates.push(directory.join("opencode.cmd"));
            }
            candidates.push(directory.join("opencode"));
        }
    }

    candidates.into_iter().find_map(|candidate| {
        let canonical = std::fs::canonicalize(candidate).ok()?;
        canonical.is_file().then_some(canonical)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_provider_key_parses_canonical_and_legacy_forms() {
        assert_eq!(
            NativeProviderKey::from_profile_key("OPENCODE"),
            Some(NativeProviderKey::OpenCode)
        );
        // The stored canonical form is upper case, but the legacy supervisor
        // literal was lower case; both must reach the OpenCode arm.
        assert_eq!(
            NativeProviderKey::from_profile_key("opencode"),
            Some(NativeProviderKey::OpenCode)
        );
        assert_eq!(
            NativeProviderKey::from_profile_key("CLAUDE_CODE"),
            Some(NativeProviderKey::ClaudeCode)
        );
        assert_eq!(
            NativeProviderKey::from_profile_key("CODEX"),
            Some(NativeProviderKey::Codex)
        );
    }

    #[test]
    fn native_provider_key_fails_closed_on_unknown() {
        for unknown in ["", "  ", "gemini", "OPEN_CODE", "claude"] {
            assert_eq!(
                NativeProviderKey::from_profile_key(unknown),
                None,
                "{unknown}"
            );
        }
    }

    fn probe_for(
        version: Option<&str>,
        readiness: OpenCodeProbeReadiness,
        host_stopped: bool,
    ) -> OpenCodeProfileProbe {
        let mut probe = OpenCodeProfileProbe::initial();
        probe.protocol_version = version.map(str::to_owned);
        probe.probe_readiness = readiness;
        probe.host_process_stopped = host_stopped;
        probe
    }

    #[test]
    fn offer_is_compatible_only_for_the_exact_2_0_32_tuple() {
        let admission = opencode_offer_admission(&probe_for(
            Some("2.0.32"),
            OpenCodeProbeReadiness::Complete,
            true,
        ));
        assert!(admission.compatible);
        assert_eq!(admission.readiness, "STARTABLE");
        assert!(admission.reason.is_none());
        assert!(opencode_profile_admission_supported(&probe_for(
            Some("2.0.32"),
            OpenCodeProbeReadiness::Complete,
            true,
        )));
    }

    #[test]
    fn offer_is_incompatible_for_each_negative() {
        // 2.0.28 is observation-only, never a compatible route.
        let older = opencode_offer_admission(&probe_for(
            Some("2.0.28"),
            OpenCodeProbeReadiness::Complete,
            true,
        ));
        assert!(!older.compatible);
        assert_eq!(older.readiness, "UNAVAILABLE");
        assert!(older.reason.is_some());

        // Unobserved version.
        assert!(
            !opencode_offer_admission(&probe_for(
                Some("1.0.0"),
                OpenCodeProbeReadiness::Complete,
                true
            ))
            .compatible
        );
        assert!(
            !opencode_offer_admission(&probe_for(None, OpenCodeProbeReadiness::Complete, true))
                .compatible
        );

        // Partial probe for the right version.
        let partial = opencode_offer_admission(&probe_for(
            Some("2.0.32"),
            OpenCodeProbeReadiness::Partial,
            true,
        ));
        assert!(!partial.compatible);
        assert_eq!(partial.readiness, "DEGRADED");

        // Host stop unobserved for the right version and complete probe.
        let no_stop = opencode_offer_admission(&probe_for(
            Some("2.0.32"),
            OpenCodeProbeReadiness::Complete,
            false,
        ));
        assert!(!no_stop.compatible);
        assert_eq!(no_stop.readiness, "DEGRADED");

        for probe in [older, partial, no_stop] {
            assert!(!probe.compatible);
        }
    }
}
