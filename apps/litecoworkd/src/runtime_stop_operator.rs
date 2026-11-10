//! Read-only Runtime stop preview and update quiescence verdict (011-T13 prerequisite).
//!
//! This is the observable half of the stop contract only. It reads durable owner records
//! (admitted Conversation turns, non-terminal Attempts, unresolved Effects, execution and
//! environment control leases, pending cleanup) and reports a verdict. It does not begin a
//! drain, fence admission or stop anything: `requestLocalRuntimeStop` stays unmounted until
//! every owner settlement hook exists.
//!
//! `QUIESCENT` is reachable only when every durable list is empty AND a live-work observer
//! (agent sessions, child processes, terminal and browser sessions) has positively reported
//! that nothing is running. No such observer exists yet, so production passes
//! `LiveWorkObservation::Unobserved` and the verdict is never `QUIESCENT` today.

use serde::Serialize;
use storage_core::RuntimeStopObservation;

/// Categories are cut at this many identifiers; the verdict is BLOCKED either way.
pub(crate) const MAX_IDS_PER_CATEGORY: usize = 200;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LiveWorkObservation {
    /// No owner hook can currently prove that live sessions and processes are absent.
    Unobserved,
    /// Every owner hook positively reported no live work. Test-only until hooks exist.
    #[cfg_attr(not(test), allow(dead_code))]
    NoneRunning,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum StopVerdict {
    Quiescent,
    Blocked,
    Unknown,
}

#[derive(Debug, Serialize)]
pub(crate) struct StopBlockerView {
    blocker_id: String,
    code: &'static str,
    safe_message: &'static str,
    resolution_hint: &'static str,
    created_at: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct RuntimeStopPreviewView {
    runtime_id: String,
    runtime_incarnation_id: String,
    observed_at: String,
    admitted_turn_ids: Vec<String>,
    admitted_attempt_ids: Vec<String>,
    unresolved_effect_ids: Vec<String>,
    execution_lease_ids: Vec<String>,
    environment_control_lease_ids: Vec<String>,
    provider_reference_ids: Vec<String>,
    pending_cleanup_ids: Vec<String>,
    verdict: StopVerdict,
    dependent_task_ids: Vec<String>,
    dependent_automation_ids: Vec<String>,
    workspace_root_ids: Vec<String>,
    shared_service_refs: Vec<String>,
    eligible_handoff_task_ids: Vec<String>,
    blockers: Vec<StopBlockerView>,
}

#[cfg(test)]
impl RuntimeStopPreviewView {
    pub(crate) fn verdict(&self) -> StopVerdict {
        self.verdict
    }
}

fn bounded(mut ids: Vec<String>) -> Vec<String> {
    ids.truncate(MAX_IDS_PER_CATEGORY);
    ids
}

pub(crate) fn build_stop_preview(
    runtime_id: &str,
    runtime_incarnation_id: &str,
    observed_at: &str,
    observation: RuntimeStopObservation,
    live: LiveWorkObservation,
) -> RuntimeStopPreviewView {
    let durable_work_present = !observation.admitted_turn_ids.is_empty()
        || !observation.admitted_attempt_ids.is_empty()
        || !observation.unresolved_effect_ids.is_empty()
        || !observation.execution_lease_ids.is_empty()
        || !observation.environment_control_lease_ids.is_empty()
        || !observation.pending_cleanup_ids.is_empty();
    let verdict = if durable_work_present {
        StopVerdict::Blocked
    } else if live == LiveWorkObservation::Unobserved {
        StopVerdict::Unknown
    } else {
        StopVerdict::Quiescent
    };

    let mut blockers = Vec::new();
    let mut push =
        |code: &'static str, safe_message: &'static str, resolution_hint: &'static str| {
            blockers.push(StopBlockerView {
                blocker_id: format!("local-runtime:{runtime_id}:stop:{}", blockers.len() + 1),
                code,
                safe_message,
                resolution_hint,
                created_at: observed_at.to_owned(),
            });
        };
    if !observation.admitted_turn_ids.is_empty() {
        push(
            "RUNTIME_STOP_BLOCKED",
            "A Conversation turn is still active.",
            "Wait for the turn to finish or stop it, then check again.",
        );
    }
    if !observation.admitted_attempt_ids.is_empty() {
        push(
            "RUNTIME_STOP_BLOCKED",
            "A Task Attempt is still active.",
            "Wait for the Attempt to settle or cancel it, then check again.",
        );
    }
    if !observation.unresolved_effect_ids.is_empty() {
        push(
            "RUNTIME_STOP_UNSETTLED",
            "An external Effect is dispatched and not yet reconciled.",
            "Let reconciliation finish before stopping or updating.",
        );
    }
    if !observation.execution_lease_ids.is_empty()
        || !observation.environment_control_lease_ids.is_empty()
    {
        push(
            "RUNTIME_STOP_UNSETTLED",
            "An execution or environment control lease is still held.",
            "Wait for the lease to be released, then check again.",
        );
    }
    if !observation.pending_cleanup_ids.is_empty() {
        push(
            "RUNTIME_STOP_UNSETTLED",
            "Cleanup work is still pending.",
            "Wait for pending cleanup to finish, then check again.",
        );
    }
    if live == LiveWorkObservation::Unobserved {
        push(
            "RUNTIME_QUIESCENCE_UNKNOWN",
            "Live agent sessions, child processes, terminal and browser sessions cannot be observed yet.",
            "Do not update or stop automatically; quiescence cannot be proven in this build.",
        );
    }

    RuntimeStopPreviewView {
        runtime_id: runtime_id.to_owned(),
        runtime_incarnation_id: runtime_incarnation_id.to_owned(),
        observed_at: observed_at.to_owned(),
        admitted_turn_ids: bounded(observation.admitted_turn_ids),
        admitted_attempt_ids: bounded(observation.admitted_attempt_ids),
        unresolved_effect_ids: bounded(observation.unresolved_effect_ids),
        execution_lease_ids: bounded(observation.execution_lease_ids),
        environment_control_lease_ids: bounded(observation.environment_control_lease_ids),
        provider_reference_ids: Vec::new(),
        pending_cleanup_ids: bounded(observation.pending_cleanup_ids),
        verdict,
        dependent_task_ids: bounded(observation.dependent_task_ids),
        dependent_automation_ids: Vec::new(),
        workspace_root_ids: Vec::new(),
        shared_service_refs: Vec::new(),
        eligible_handoff_task_ids: Vec::new(),
        blockers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: &str = "2026-11-10T00:00:00Z";

    fn preview(
        observation: RuntimeStopObservation,
        live: LiveWorkObservation,
    ) -> RuntimeStopPreviewView {
        build_stop_preview("rt", "inc", NOW, observation, live)
    }

    fn ids(value: &str) -> Vec<String> {
        vec![value.to_owned()]
    }

    #[test]
    fn idle_runtime_without_live_observer_is_unknown_never_quiescent() {
        let view = preview(
            RuntimeStopObservation::default(),
            LiveWorkObservation::Unobserved,
        );
        assert_eq!(view.verdict(), StopVerdict::Unknown);
        assert_eq!(view.blockers.len(), 1);
        assert_eq!(view.blockers[0].code, "RUNTIME_QUIESCENCE_UNKNOWN");
    }

    #[test]
    fn idle_runtime_with_positive_live_observation_is_quiescent() {
        let view = preview(
            RuntimeStopObservation::default(),
            LiveWorkObservation::NoneRunning,
        );
        assert_eq!(view.verdict(), StopVerdict::Quiescent);
        assert!(view.blockers.is_empty());
    }

    #[test]
    fn each_durable_category_blocks_and_is_listed() {
        let cases: Vec<(RuntimeStopObservation, &str)> = vec![
            (
                RuntimeStopObservation {
                    admitted_turn_ids: ids("t"),
                    ..Default::default()
                },
                "RUNTIME_STOP_BLOCKED",
            ),
            (
                RuntimeStopObservation {
                    admitted_attempt_ids: ids("a"),
                    ..Default::default()
                },
                "RUNTIME_STOP_BLOCKED",
            ),
            (
                RuntimeStopObservation {
                    unresolved_effect_ids: ids("e"),
                    ..Default::default()
                },
                "RUNTIME_STOP_UNSETTLED",
            ),
            (
                RuntimeStopObservation {
                    execution_lease_ids: ids("l"),
                    ..Default::default()
                },
                "RUNTIME_STOP_UNSETTLED",
            ),
            (
                RuntimeStopObservation {
                    environment_control_lease_ids: ids("c"),
                    ..Default::default()
                },
                "RUNTIME_STOP_UNSETTLED",
            ),
            (
                RuntimeStopObservation {
                    pending_cleanup_ids: ids("p"),
                    ..Default::default()
                },
                "RUNTIME_STOP_UNSETTLED",
            ),
        ];
        for (observation, code) in cases {
            // Even a positive live observation cannot override durable unsettled work.
            let view = preview(observation, LiveWorkObservation::NoneRunning);
            assert_eq!(view.verdict(), StopVerdict::Blocked);
            assert_eq!(view.blockers[0].code, code);
        }
    }

    #[test]
    fn blocked_and_unobserved_report_both_reasons() {
        let view = preview(
            RuntimeStopObservation {
                admitted_turn_ids: ids("t"),
                ..Default::default()
            },
            LiveWorkObservation::Unobserved,
        );
        assert_eq!(view.verdict(), StopVerdict::Blocked);
        let codes: Vec<_> = view.blockers.iter().map(|b| b.code).collect();
        assert_eq!(
            codes,
            ["RUNTIME_STOP_BLOCKED", "RUNTIME_QUIESCENCE_UNKNOWN"]
        );
    }

    #[test]
    fn lists_are_bounded_and_serialize_with_screaming_verdict() {
        let many: Vec<String> = (0..500).map(|i| format!("t{i}")).collect();
        let view = preview(
            RuntimeStopObservation {
                admitted_turn_ids: many,
                ..Default::default()
            },
            LiveWorkObservation::Unobserved,
        );
        assert_eq!(view.admitted_turn_ids.len(), MAX_IDS_PER_CATEGORY);
        let json = serde_json::to_value(&view).expect("serialize");
        assert_eq!(json["verdict"], "BLOCKED");
        assert_eq!(json["provider_reference_ids"], serde_json::json!([]));
    }
}
