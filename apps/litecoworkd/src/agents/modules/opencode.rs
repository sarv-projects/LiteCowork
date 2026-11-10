//! OpenCode 2.0.32 AgentModule: manifest and AgentControlDescriptor producer.
//!
//! The descriptor is built only from a fresh bounded probe of the exact accepted
//! CLI version. Nothing is inferred from a catalog entry, a Runtime offer or a
//! brand name. Authentication stays UNKNOWN until the native sign-in observation
//! (002-T07) exists, and the catalog of provider and model ids is display-only
//! option data, never connected-provider or entitlement evidence.

use crate::agents::control_descriptor::{
    AgentAuthDescriptor, AgentAuthState, AgentCapabilities, AgentConfigurationDescriptor,
    AgentControlDescriptor, AgentEnvironmentCapabilities, AgentExtensionCapabilities,
    AgentInputCapabilities, AgentInputSurfaceDescriptor, AgentInstallationObservation, AgentLimits,
    AgentNativeSurfaceDescriptor, AgentReportingCapabilities, AgentSessionCapabilities,
    AgentSessionOption, AgentSessionOptionDescriptor, InstallationObservationState,
    ModelSwitchCapability, SessionOptionMutableScope, SessionOptionSemanticHint,
};
use crate::agents::module_registry::{
    AgentModuleLifecycleFeatures, AgentModuleManifest, OperatingArchitecture, OperatingSystem,
};
use crate::agents::opencode_profile_probe::OpenCodeProfileProbe;
use sha2::{Digest, Sha256};

pub(crate) const OPENCODE_MODULE_ID: &str = "opencode";
pub(crate) const OPENCODE_MODULE_VERSION: &str = "2.0.32";
pub(crate) const OPENCODE_REGISTRY_AGENT_ID: &str = "opencode";
/// Largest serialized model enum kept in the option schema (the validator bound is
/// 16 KiB per schema; stay well below it).
const MAX_MODEL_ENUM_BYTES: usize = 12 * 1024;

pub(crate) fn manifest() -> AgentModuleManifest {
    AgentModuleManifest {
        module_id: OPENCODE_MODULE_ID.to_owned(),
        module_version: OPENCODE_MODULE_VERSION.to_owned(),
        descriptor_api_version: 1,
        registry_agent_ids: vec![OPENCODE_REGISTRY_AGENT_ID.to_owned()],
        display_name: "OpenCode".to_owned(),
        distribution_kinds: vec!["NPM".to_owned(), "BINARY".to_owned()],
        supported_os: vec![OperatingSystem::Windows, OperatingSystem::Linux],
        supported_architectures: vec![OperatingArchitecture::X86_64],
        // V1 has no install or update: the module only discovers and (later) reads
        // native authentication. Every lifecycle flag stays false until exercised.
        lifecycle_features: AgentModuleLifecycleFeatures {
            discover_installation: true,
            install: false,
            update: false,
            uninstall: false,
            native_auth: false,
            secure_credential_slots: false,
            open_native_config: false,
        },
        descriptor_ttl_seconds: 900,
        probe_budget_per_minute: 3,
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum DescriptorBuildError {
    /// The probe did not observe exactly the accepted CLI version.
    UnsupportedVersion,
    /// The built descriptor failed the shared validator (programming error).
    Invalid,
}

/// Builds and seals a validated descriptor from one bounded probe observation.
pub(crate) fn build_descriptor(
    probe: &OpenCodeProfileProbe,
    runtime_id: &str,
    runtime_incarnation_id: &str,
    observed_at: time::OffsetDateTime,
    ttl_seconds: u32,
) -> Result<AgentControlDescriptor, DescriptorBuildError> {
    if probe.protocol_version.as_deref() != Some(OPENCODE_MODULE_VERSION) {
        return Err(DescriptorBuildError::UnsupportedVersion);
    }
    let format = &time::format_description::well_known::Rfc3339;
    let observed = observed_at
        .replace_nanosecond(0)
        .map_err(|_| DescriptorBuildError::Invalid)?
        .format(format)
        .map_err(|_| DescriptorBuildError::Invalid)?;
    let expires = (observed_at + time::Duration::seconds(i64::from(ttl_seconds)))
        .replace_nanosecond(0)
        .map_err(|_| DescriptorBuildError::Invalid)?
        .format(format)
        .map_err(|_| DescriptorBuildError::Invalid)?;

    let session_options = session_options(probe)?;
    let mut descriptor = AgentControlDescriptor {
        descriptor_digest: format!("sha256:{}", "0".repeat(64)),
        descriptor_api_version: 1,
        registry_agent_id: OPENCODE_REGISTRY_AGENT_ID.to_owned(),
        module_id: OPENCODE_MODULE_ID.to_owned(),
        module_version: OPENCODE_MODULE_VERSION.to_owned(),
        harness_version: probe.protocol_version.clone(),
        protocol_version: probe.protocol_version.clone(),
        installation: AgentInstallationObservation {
            registry_agent_id: OPENCODE_REGISTRY_AGENT_ID.to_owned(),
            module_id: OPENCODE_MODULE_ID.to_owned(),
            runtime_id: runtime_id.to_owned(),
            runtime_incarnation_id: runtime_incarnation_id.to_owned(),
            state: InstallationObservationState::Installed,
            installed_version: probe.protocol_version.clone(),
            available_version: None,
            installation_source: None,
            observed_at: observed.clone(),
            expires_at: expires.clone(),
        },
        auth: AgentAuthDescriptor {
            state: AgentAuthState::Unknown,
            account_label: None,
            methods: Vec::new(),
            credential_slots: Vec::new(),
        },
        configuration: AgentConfigurationDescriptor {
            schema_digest: sha256_of(b"{}"),
            fields: Vec::new(),
            native_config_targets: Vec::new(),
        },
        session_options,
        // Text only. No native command list is declared: OpenCode 2.0.32 exposes no
        // list the module can read and the supervisor can execute.
        input_surface: AgentInputSurfaceDescriptor {
            text: true,
            images: false,
            files: false,
            resources: false,
            slash_commands: Vec::new(),
            reference_kinds: Vec::new(),
            supports_command_discovery: false,
            supports_reference_discovery: false,
        },
        native_surface: AgentNativeSurfaceDescriptor {
            native_config_preserved: true,
            native_extensions_preserved: true,
            native_subagents_preserved: true,
            summary: None,
        },
        // Matches `opencode_endpoint_capabilities()`: only input.text is true.
        capabilities: AgentCapabilities {
            protocol: "OPENCODE_SERVER".to_owned(),
            protocol_version: probe.protocol_version.clone(),
            session: AgentSessionCapabilities {
                resume: false,
                steer: false,
                interrupt: false,
                cancel: false,
                fork: false,
                model_switch: ModelSwitchCapability::Unsupported,
            },
            input: AgentInputCapabilities {
                text: true,
                image: false,
                file: false,
                resources: false,
            },
            extension: AgentExtensionCapabilities {
                mcp_stdio: false,
                mcp_http: false,
                skills: false,
                plugins: false,
                dynamic_attach: false,
            },
            reporting: AgentReportingCapabilities {
                tool_calls: false,
                plan: false,
                usage: false,
                native_subagents: false,
                approvals: false,
            },
            environment: AgentEnvironmentCapabilities {
                cwd: false,
                extra_directories: false,
            },
            limits: AgentLimits {
                max_context_hint: None,
                max_concurrent_sessions: None,
            },
            host_instruction_delivery: None,
        },
        observed_at: observed,
        expires_at: expires,
    };
    descriptor.descriptor_digest = descriptor
        .computed_digest()
        .map_err(|_| DescriptorBuildError::Invalid)?;
    descriptor
        .validate()
        .map_err(|_| DescriptorBuildError::Invalid)?;
    Ok(descriptor)
}

/// Model option from the probe's configured providers as `provider/model` ids, so no
/// cross-provider mixing is possible. Absent when the probe observed no models.
fn session_options(
    probe: &OpenCodeProfileProbe,
) -> Result<Option<AgentSessionOptionDescriptor>, DescriptorBuildError> {
    let mut values: Vec<String> = Vec::new();
    let mut bytes = 0_usize;
    'providers: for provider in &probe.configured_providers {
        for model in &provider.models {
            let value = format!("{}/{}", provider.id, model.id);
            bytes += value.len() + 4;
            if bytes > MAX_MODEL_ENUM_BYTES {
                break 'providers;
            }
            values.push(value);
        }
    }
    if values.is_empty() {
        return Ok(None);
    }
    let options = vec![AgentSessionOption {
        option_id: "model".to_owned(),
        display_name: "Model".to_owned(),
        semantic_hint: Some(SessionOptionSemanticHint::Model),
        value_schema: serde_json::json!({"type": "string", "enum": values}),
        // Session-model selection is NOT_QUALIFIED for the supervisor path, so the
        // option is declared for display and future sessions only.
        mutable_scope: SessionOptionMutableScope::NewSession,
    }];
    let encoded = serde_json::to_vec(&options).map_err(|_| DescriptorBuildError::Invalid)?;
    Ok(Some(AgentSessionOptionDescriptor {
        descriptor_digest: sha256_of(&encoded),
        options,
    }))
}

fn sha256_of(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::module_registry::ModuleSupportAssessment;
    use crate::agents::modules::default_module_registry;
    use crate::agents::opencode_profile_probe::{
        OpenCodeModelOption, OpenCodeObservation, OpenCodeProbeReadiness, OpenCodeProviderOption,
    };

    const FIXTURE: &str =
        include_str!("../../../tests/fixtures/agent-modules/opencode-2.0.32.json");

    fn probe(version: Option<&str>) -> OpenCodeProfileProbe {
        let mut probe = OpenCodeProfileProbe::initial();
        probe.protocol_version = version.map(str::to_owned);
        probe.probe_readiness = OpenCodeProbeReadiness::Complete;
        probe.reported_model_catalog = OpenCodeObservation::Observed;
        probe.host_process_stopped = true;
        probe.configured_providers = vec![OpenCodeProviderOption {
            id: "anthropic".to_owned(),
            display_name: None,
            models: vec![
                OpenCodeModelOption {
                    id: "sonnet".to_owned(),
                    display_name: None,
                },
                OpenCodeModelOption {
                    id: "opus".to_owned(),
                    display_name: None,
                },
            ],
        }];
        probe
    }

    fn now() -> time::OffsetDateTime {
        time::OffsetDateTime::from_unix_timestamp(1_800_000_000).unwrap()
    }

    #[test]
    fn module_registry_selects_opencode_only_for_the_exact_supported_runtime() {
        let registry = default_module_registry();
        for (os, arch, expected) in [
            (
                OperatingSystem::Windows,
                OperatingArchitecture::X86_64,
                ModuleSupportAssessment::Supported,
            ),
            (
                OperatingSystem::Linux,
                OperatingArchitecture::X86_64,
                ModuleSupportAssessment::Supported,
            ),
            (
                OperatingSystem::Macos,
                OperatingArchitecture::X86_64,
                ModuleSupportAssessment::UnsupportedRuntime,
            ),
            (
                OperatingSystem::Linux,
                OperatingArchitecture::Aarch64,
                ModuleSupportAssessment::UnsupportedRuntime,
            ),
        ] {
            assert_eq!(
                registry
                    .assess_registry_agent("opencode", os, arch, 1)
                    .unwrap(),
                expected
            );
        }
        assert_eq!(
            registry
                .assess_registry_agent(
                    "some-other-agent",
                    OperatingSystem::Linux,
                    OperatingArchitecture::X86_64,
                    1
                )
                .unwrap(),
            ModuleSupportAssessment::NoAdapter
        );
        assert_eq!(registry.get("opencode").unwrap().module_version, "2.0.32");
    }

    #[test]
    fn module_registry_rejects_a_duplicate_opencode_claim() {
        let mut registry = default_module_registry();
        assert!(registry.register(manifest()).is_err());
    }

    #[test]
    fn module_registry_manifest_matches_the_pinned_conformance_fixture() {
        let fixture: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
        let manifest = manifest();
        assert_eq!(fixture["module_id"], manifest.module_id);
        assert_eq!(fixture["module_version"], manifest.module_version);
        assert_eq!(
            fixture["descriptor_ttl_seconds"],
            manifest.descriptor_ttl_seconds
        );
        assert_eq!(
            fixture["probe_budget_per_minute"],
            manifest.probe_budget_per_minute
        );
        assert_eq!(
            fixture["registry_agent_ids"][0],
            manifest.registry_agent_ids[0]
        );
        // No lifecycle feature may be claimed that the fixture does not declare.
        let features = &fixture["lifecycle_features"];
        assert_eq!(features["install"], false);
        assert_eq!(features["update"], false);
        assert_eq!(features["native_auth"], false);
        assert_eq!(
            features["discover_installation"],
            manifest.lifecycle_features.discover_installation
        );
        assert_eq!(fixture["credential_slots"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn module_registry_fixture_without_ttl_or_budget_fails_validation() {
        for missing in ["descriptor_ttl_seconds", "probe_budget_per_minute"] {
            let mut fixture: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
            fixture.as_object_mut().unwrap().remove(missing);
            assert!(fixture.get(missing).is_none());
            assert!(
                fixture_declares_budget(&fixture).is_err(),
                "{missing} must be required"
            );
        }
        assert!(fixture_declares_budget(&serde_json::from_str(FIXTURE).unwrap()).is_ok());
    }

    fn fixture_declares_budget(fixture: &serde_json::Value) -> Result<(), ()> {
        let ttl = fixture["descriptor_ttl_seconds"].as_u64().ok_or(())?;
        let budget = fixture["probe_budget_per_minute"].as_u64().ok_or(())?;
        if (1..=900).contains(&ttl) && (1..=3).contains(&budget) {
            Ok(())
        } else {
            Err(())
        }
    }

    #[test]
    fn manifest_cannot_exceed_the_host_ttl_or_probe_ceiling() {
        let mut registry = crate::agents::module_registry::AgentModuleRegistry::default();
        let mut too_long = manifest();
        too_long.descriptor_ttl_seconds = 901;
        assert!(registry.register(too_long).is_err());
        let mut too_chatty = manifest();
        too_chatty.probe_budget_per_minute = 4;
        assert!(registry.register(too_chatty).is_err());
        let mut lower = manifest();
        lower.descriptor_ttl_seconds = 300;
        lower.probe_budget_per_minute = 1;
        assert!(registry.register(lower).is_ok());
    }

    #[test]
    fn descriptor_is_built_validated_and_declares_only_what_the_supervisor_sends() {
        let descriptor = build_descriptor(&probe(Some("2.0.32")), "rt", "rti", now(), 900).unwrap();
        descriptor.validate().unwrap();
        assert!(descriptor.capabilities.input.text);
        assert!(!descriptor.capabilities.input.file && !descriptor.capabilities.input.image);
        assert!(descriptor.input_surface.slash_commands.is_empty());
        assert_eq!(descriptor.auth.state, AgentAuthState::Unknown);
        assert!(descriptor.auth.credential_slots.is_empty());
        let options = descriptor.session_options.as_ref().unwrap();
        assert_eq!(
            options.options[0].value_schema["enum"],
            serde_json::json!(["anthropic/sonnet", "anthropic/opus"])
        );
        // No executable path, native handle or secret-shaped text in the payload.
        let body = serde_json::to_string(&descriptor).unwrap();
        for forbidden in ["/usr", "\\Users", "apikey", "secret", "password"] {
            assert!(
                !body
                    .to_ascii_lowercase()
                    .contains(&forbidden.to_ascii_lowercase()),
                "{forbidden}"
            );
        }
        assert!(descriptor.is_fresh_at((now() + time::Duration::seconds(899)).into()));
        assert!(!descriptor.is_fresh_at((now() + time::Duration::seconds(900)).into()));
    }

    #[test]
    fn descriptor_is_refused_for_any_version_other_than_the_pinned_one() {
        for version in [None, Some("2.0.28"), Some("2.0.26"), Some("2.0.33")] {
            assert_eq!(
                build_descriptor(&probe(version), "rt", "rti", now(), 900).unwrap_err(),
                DescriptorBuildError::UnsupportedVersion
            );
        }
    }

    #[test]
    fn descriptor_without_observed_models_declares_no_session_options() {
        let mut bare = probe(Some("2.0.32"));
        bare.configured_providers.clear();
        let descriptor = build_descriptor(&bare, "rt", "rti", now(), 900).unwrap();
        assert!(descriptor.session_options.is_none());
    }
}
