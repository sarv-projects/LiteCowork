//! The only entry the Operator uses to start native work for an admitted turn
//! (001-T20, SPEC L-07).
//!
//! The Operator admits a turn (the durable USER message, OPEN turn and dispatch intent
//! commit atomically first, AC-001) and returns 202. It then hands the turn to this
//! coordinator, which drives the `AgentSessionSupervisor` off the request path. The
//! supervisor claims the durable dispatch intent before reserving a session, and the
//! startup recovery sweep owns any intent a crash left behind (AC-022).

use std::sync::Arc;

use storage_core::conversation::ConversationTurn;

use super::supervisor::AgentSessionSupervisor;

#[derive(Clone)]
pub(crate) struct AgentTurnCoordinator {
    supervisor: Arc<AgentSessionSupervisor>,
}

impl AgentTurnCoordinator {
    pub(crate) fn new(supervisor: Arc<AgentSessionSupervisor>) -> Self {
        Self { supervisor }
    }

    /// Starts dispatch for an admitted (or retried) turn without blocking the caller.
    /// A dispatch error has already settled the turn FAILED inside the supervisor; it is
    /// reported here only as a redacted diagnostic.
    pub(crate) fn dispatch_admitted_turn(
        &self,
        principal_id: &str,
        workspace_id: &str,
        conversation_id: &str,
        turn: ConversationTurn,
        correlation_id: &str,
    ) {
        let supervisor = Arc::clone(&self.supervisor);
        let principal_id = principal_id.to_owned();
        let workspace_id = workspace_id.to_owned();
        let conversation_id = conversation_id.to_owned();
        let correlation_id = correlation_id.to_owned();
        let turn_id = turn.turn_id.clone();
        tokio::spawn(async move {
            if let Err(error) = supervisor
                .dispatch_turn(
                    &principal_id,
                    &workspace_id,
                    &conversation_id,
                    turn,
                    &correlation_id,
                )
                .await
            {
                eprintln!("{}", dispatch_diagnostic(&turn_id, &correlation_id, &error));
            }
        });
    }
}

/// A diagnostic line carrying only correlation identifiers and a coarse error class.
/// The error text may embed provider detail, so none of it is copied: only the leading
/// label (letters, digits, spaces, `_` and `-`, at most 48 characters, cut at the first
/// `:` or `{`) is kept.
pub(crate) fn dispatch_diagnostic(turn_id: &str, correlation_id: &str, error: &str) -> String {
    let class: String = error
        .split([':', '{', '('])
        .next()
        .unwrap_or("")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '-'))
        .take(48)
        .collect();
    let class = class.trim();
    format!(
        "conversation dispatch failed turn_id={turn_id} correlation_id={correlation_id} class={}",
        if class.is_empty() { "UNKNOWN" } else { class }
    )
}

#[cfg(test)]
mod tests {
    use super::dispatch_diagnostic;

    #[test]
    fn conversation_turn_dispatch_diagnostic_carries_ids_and_class_but_no_detail() {
        let line = dispatch_diagnostic(
            "turn-1",
            "corr-1",
            "Start session error: Database(\"prompt: tell me the secret ses_native_handle\")",
        );
        assert_eq!(
            line,
            "conversation dispatch failed turn_id=turn-1 correlation_id=corr-1 class=Start session error"
        );
        assert!(!line.contains("secret") && !line.contains("ses_native_handle"));
        assert!(dispatch_diagnostic("t", "c", "").ends_with("class=UNKNOWN"));
    }
}
