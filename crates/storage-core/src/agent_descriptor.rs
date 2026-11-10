//! Runtime-local AgentControlDescriptor persistence (specs/002 T04).
//!
//! Rows live in the existing `agent_control_descriptors` table (migration 16). The
//! store keeps the validated, non-secret descriptor JSON with its digest and
//! freshness window. It is Runtime-local: no Workspace id, executable path, native
//! handle or credential is stored. Validation of the descriptor itself belongs to the
//! daemon's AgentModule layer; this layer enforces size, the envelope shape, the
//! compare-and-swap on the prior digest and TTL-based currency.

use crate::StoreError;
use serde::{Deserialize, Serialize};

/// Largest stored descriptor, matching the daemon validator bound.
pub const AGENT_CONTROL_DESCRIPTOR_MAX_BYTES: usize = 256 * 1024;
/// Descriptor freshness window in seconds (B-12).
pub const AGENT_CONTROL_DESCRIPTOR_TTL_SECONDS: i64 = 900;
/// Default probe budget per module per minute (B-12). A module may lower it.
pub const AGENT_DESCRIPTOR_PROBES_PER_MINUTE: u32 = 3;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentControlDescriptorRecord {
    pub registry_agent_id: String,
    pub runtime_id: String,
    pub runtime_incarnation_id: String,
    pub module_id: String,
    pub module_version: String,
    pub descriptor_api_version: u32,
    pub descriptor_digest: String,
    pub descriptor_json: String,
    pub observed_at: String,
    pub expires_at: String,
}

/// Identifies which stored descriptor stream an enable must match.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentDescriptorGate {
    pub registry_agent_id: String,
    pub runtime_id: String,
    pub runtime_incarnation_id: String,
    pub module_version: String,
}

pub trait AgentDescriptorStore: Send + Sync {
    /// Appends a descriptor revision for the stream
    /// `(registry_agent_id, runtime_id, runtime_incarnation_id, module_version)`.
    /// `expected_prior_digest` is the digest of the latest stored revision the caller
    /// observed (None for the first write); a mismatch returns a version conflict and
    /// writes nothing.
    fn put_agent_control_descriptor(
        &self,
        record: AgentControlDescriptorRecord,
        expected_prior_digest: Option<String>,
    ) -> Result<(), StoreError>;

    /// Returns the latest stored revision regardless of freshness. Only a writer uses
    /// this, to obtain the digest for the compare-and-swap; it is never evidence of
    /// currency.
    fn latest_agent_control_descriptor(
        &self,
        gate: &AgentDescriptorGate,
    ) -> Result<Option<AgentControlDescriptorRecord>, StoreError>;

    /// Returns the latest revision of the stream only while `now < expires_at`.
    fn current_agent_control_descriptor(
        &self,
        gate: &AgentDescriptorGate,
        now: &str,
    ) -> Result<Option<AgentControlDescriptorRecord>, StoreError>;
}

pub fn validate_agent_control_descriptor_record(
    record: &AgentControlDescriptorRecord,
) -> Result<(), String> {
    let id_ok = |value: &str, max: usize| {
        !value.is_empty() && value.len() <= max && !value.chars().any(char::is_control)
    };
    if !id_ok(&record.registry_agent_id, 128)
        || !id_ok(&record.runtime_id, 256)
        || !id_ok(&record.runtime_incarnation_id, 256)
        || !id_ok(&record.module_id, 128)
        || !id_ok(&record.module_version, 128)
        || record.descriptor_api_version == 0
    {
        return Err("AgentControlDescriptor identity is invalid".to_owned());
    }
    let digest_ok = record.descriptor_digest.len() == 71
        && record
            .descriptor_digest
            .strip_prefix("sha256:")
            .is_some_and(|hex| {
                hex.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            });
    if !digest_ok {
        return Err("AgentControlDescriptor digest is invalid".to_owned());
    }
    if record.descriptor_json.len() < 2
        || record.descriptor_json.len() > AGENT_CONTROL_DESCRIPTOR_MAX_BYTES
        || serde_json::from_str::<serde_json::Value>(&record.descriptor_json)
            .map_or(true, |value| !value.is_object())
    {
        return Err("AgentControlDescriptor body is invalid or too large".to_owned());
    }
    if record.observed_at.is_empty()
        || record.expires_at.is_empty()
        || record.observed_at >= record.expires_at
    {
        return Err("AgentControlDescriptor freshness window is invalid".to_owned());
    }
    Ok(())
}
