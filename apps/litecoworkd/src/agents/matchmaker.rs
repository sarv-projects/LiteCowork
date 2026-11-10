//! Agent Matchmaker ranking (specs/002 T18, AC-019).
//!
//! Advisory only. Given a plain-language job description and the user's bound
//! agents, rank the eligible ones by what each agent's fresh, module-declared
//! descriptor says it supports. The ranking never selects, dispatches or
//! overrides anything: the caller still chooses the agent. A candidate that is
//! disabled, not lead-eligible, or has no fresh descriptor gets no rank.
#![allow(
    dead_code,
    reason = "advisory ranking engine is exercised by tests until an Operator route and UI surface it"
)]

use crate::agents::control_descriptor::{AgentAuthState, AgentControlDescriptor};
use std::time::SystemTime;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Need {
    Images,
    Files,
    Resources,
    Subagents,
    Extensions,
    Fork,
    Commands,
}

impl Need {
    fn label(self) -> &'static str {
        match self {
            Self::Images => "image input",
            Self::Files => "file input",
            Self::Resources => "Workspace resource input",
            Self::Subagents => "native subagents",
            Self::Extensions => "MCP, skills or plugins",
            Self::Fork => "forking a session",
            Self::Commands => "native commands",
        }
    }

    fn supported_by(self, descriptor: &AgentControlDescriptor) -> bool {
        let capabilities = &descriptor.capabilities;
        match self {
            Self::Images => descriptor.input_surface.images && capabilities.input.image,
            Self::Files => descriptor.input_surface.files && capabilities.input.file,
            Self::Resources => descriptor.input_surface.resources && capabilities.input.resources,
            Self::Subagents => capabilities.reporting.native_subagents,
            Self::Extensions => {
                let extension = &capabilities.extension;
                extension.mcp_stdio || extension.mcp_http || extension.skills || extension.plugins
            }
            Self::Fork => capabilities.session.fork,
            Self::Commands => !descriptor.input_surface.slash_commands.is_empty(),
        }
    }
}

/// Needs are read from the job text with fixed keywords. Unrecognised text yields
/// no needs and so no preference between agents.
fn needs_from(job: &str) -> Vec<Need> {
    let text = job.to_lowercase();
    let table: [(&[&str], Need); 7] = [
        (&["image", "screenshot", "photo", "diagram"], Need::Images),
        (&["file", "attachment", "document", "pdf"], Need::Files),
        (&["resource", "workspace", "folder"], Need::Resources),
        (&["subagent", "parallel", "delegate"], Need::Subagents),
        (&["mcp", "plugin", "skill", "extension"], Need::Extensions),
        (&["fork", "branch the session"], Need::Fork),
        (&["slash command", "/review", "command"], Need::Commands),
    ];
    let mut needs: Vec<Need> = table
        .iter()
        .filter(|(words, _)| words.iter().any(|word| text.contains(word)))
        .map(|(_, need)| *need)
        .collect();
    needs.sort();
    needs.dedup();
    needs
}

#[derive(Clone, Debug)]
pub(crate) struct MatchmakerCandidate {
    pub agent_binding_id: String,
    pub display_name: String,
    pub enabled: bool,
    pub lead_eligible: bool,
    /// The current unexpired descriptor, or None when it is missing or stale.
    pub descriptor: Option<AgentControlDescriptor>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct RankedAgent {
    pub agent_binding_id: String,
    pub rank: usize,
    pub score: usize,
    pub reason: String,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ExcludedAgent {
    pub agent_binding_id: String,
    pub reason: &'static str,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct MatchmakerResult {
    pub ranked: Vec<RankedAgent>,
    pub excluded: Vec<ExcludedAgent>,
    /// Always true: the ranking is a suggestion and never auto-dispatches.
    pub advisory: bool,
}

pub(crate) fn rank_agents(
    job_description: &str,
    candidates: &[MatchmakerCandidate],
    now: SystemTime,
) -> MatchmakerResult {
    let needs = needs_from(job_description);
    let mut scored: Vec<(usize, &MatchmakerCandidate, String)> = Vec::new();
    let mut excluded = Vec::new();
    for candidate in candidates {
        let reason = if !candidate.enabled {
            Some("This agent binding is disabled.")
        } else if !candidate.lead_eligible {
            Some("This agent is not eligible to lead.")
        } else {
            match &candidate.descriptor {
                None => Some("This agent has no fresh descriptor, so it cannot be ranked."),
                Some(descriptor) if !descriptor.is_fresh_at(now) => {
                    Some("This agent's descriptor is stale, so it cannot be ranked.")
                }
                Some(_) => None,
            }
        };
        if let Some(reason) = reason {
            excluded.push(ExcludedAgent {
                agent_binding_id: candidate.agent_binding_id.clone(),
                reason,
            });
            continue;
        }
        let Some(descriptor) = candidate.descriptor.as_ref() else {
            continue;
        };
        let supported: Vec<&str> = needs
            .iter()
            .filter(|need| need.supported_by(descriptor))
            .map(|need| need.label())
            .collect();
        let missing: Vec<&str> = needs
            .iter()
            .filter(|need| !need.supported_by(descriptor))
            .map(|need| need.label())
            .collect();
        let mut reason = if needs.is_empty() {
            format!(
                "{} is eligible. The job text names no capability to compare.",
                candidate.display_name
            )
        } else if supported.is_empty() {
            format!(
                "{} declares none of the capabilities this job mentions ({}).",
                candidate.display_name,
                missing.join(", ")
            )
        } else {
            format!(
                "{} declares {}.",
                candidate.display_name,
                supported.join(", ")
            )
        };
        if !missing.is_empty() && !supported.is_empty() {
            reason.push_str(&format!(" It does not declare {}.", missing.join(", ")));
        }
        if descriptor.auth.state != AgentAuthState::Authenticated {
            reason.push_str(" Sign-in is not confirmed.");
        }
        scored.push((supported.len(), candidate, reason));
    }
    // Higher score first; ties keep the caller's order so ranking is deterministic.
    scored.sort_by_key(|entry| std::cmp::Reverse(entry.0));
    let ranked = scored
        .into_iter()
        .enumerate()
        .map(|(index, (score, candidate, reason))| RankedAgent {
            agent_binding_id: candidate.agent_binding_id.clone(),
            rank: index + 1,
            score,
            reason,
        })
        .collect();
    MatchmakerResult {
        ranked,
        excluded,
        advisory: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::modules::opencode;
    use crate::agents::opencode_profile_probe::{OpenCodeProbeReadiness, OpenCodeProfileProbe};

    fn descriptor(files: bool) -> AgentControlDescriptor {
        let mut probe = OpenCodeProfileProbe::initial();
        probe.protocol_version = Some("2.0.32".to_owned());
        probe.probe_readiness = OpenCodeProbeReadiness::Complete;
        probe.host_process_stopped = true;
        let at = time::OffsetDateTime::now_utc();
        let mut descriptor = opencode::build_descriptor(&probe, "runtime", "incarnation", at, 900)
            .expect("descriptor builds");
        if files {
            descriptor.input_surface.files = true;
            descriptor.capabilities.input.file = true;
            descriptor.descriptor_digest = descriptor.computed_digest().expect("digest");
            descriptor.validate().expect("valid");
        }
        descriptor
    }

    fn candidate(id: &str, files: bool) -> MatchmakerCandidate {
        MatchmakerCandidate {
            agent_binding_id: id.to_owned(),
            display_name: id.to_owned(),
            enabled: true,
            lead_eligible: true,
            descriptor: Some(descriptor(files)),
        }
    }

    #[test]
    fn agent_matchmaker_ranks_by_declared_capability_with_a_reason_each() {
        let result = rank_agents(
            "Summarize this PDF file",
            &[candidate("plain", false), candidate("files", true)],
            SystemTime::now(),
        );
        assert!(result.advisory);
        assert_eq!(result.ranked[0].agent_binding_id, "files");
        assert_eq!(result.ranked[0].rank, 1);
        assert!(result.ranked[0].reason.contains("file input"));
        assert_eq!(result.ranked[1].agent_binding_id, "plain");
        assert!(result.ranked[1].reason.contains("declares none"));
        assert!(result.ranked.iter().all(|entry| !entry.reason.is_empty()));
    }

    #[test]
    fn agent_matchmaker_excludes_disabled_ineligible_and_stale_candidates() {
        let mut disabled = candidate("disabled", true);
        disabled.enabled = false;
        let mut ineligible = candidate("ineligible", true);
        ineligible.lead_eligible = false;
        let mut stale = candidate("stale", true);
        stale.descriptor = Some(descriptor(true));
        let mut missing = candidate("missing", true);
        missing.descriptor = None;
        let far_future = SystemTime::now() + std::time::Duration::from_secs(7200);
        let result = rank_agents(
            "read a file",
            &[disabled, ineligible, stale, missing],
            far_future,
        );
        assert!(result.ranked.is_empty());
        assert_eq!(result.excluded.len(), 4);
        assert!(result.excluded[2].reason.contains("stale"));
    }

    #[test]
    fn agent_matchmaker_ties_keep_the_callers_order_and_never_pick_for_the_user() {
        let result = rank_agents(
            "do something",
            &[candidate("b", false), candidate("a", false)],
            SystemTime::now(),
        );
        let ids: Vec<_> = result
            .ranked
            .iter()
            .map(|r| r.agent_binding_id.as_str())
            .collect();
        assert_eq!(ids, ["b", "a"]);
        assert!(result.advisory);
        assert!(result.ranked[0].reason.contains("names no capability"));
    }
}
