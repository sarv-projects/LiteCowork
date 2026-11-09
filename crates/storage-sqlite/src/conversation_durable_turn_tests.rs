//! Conversation durable turn storage: dispatch outbox, single-flight admission,
//! final-response commit identity and fault injection (001-T02, T10, T11).
use super::*;
use storage_core::conversation::{
    CommitConversationAgentResponse, ConversationDispatchState,
    ConversationMessageRecord, ConversationTurnStatus, ConversationTurnStore,
};
use storage_core::{ActivateConversationAgentSession, AgentSessionStore, WorkspaceCreateRequest};

struct Activated {
    fixture: Fixture,
    store: crate::conversations::SqliteConversationStore,
    conversation_id: String,
    turn_id: String,
    session_id: String,
}

fn activated() -> Activated {
    let (fixture, _reserved, conversation_id, turn_id, session_id, host_id) =
        conversation_activation_fixture();
    fixture
        .store
        .activate_conversation_agent_session(ActivateConversationAgentSession {
            principal_id: OWNER_ID.to_owned(),
            workspace_id: WORKSPACE_ID.to_owned(),
            agent_session_id: session_id.clone(),
            expected_session_version: 1,
            expected_turn_version: 1,
            occurred_at: NOW.to_owned(),
            host_instance_id: host_id,
            native_session_ref: Some("ephemeral-native-session-token".to_owned()),
            session_event: draft(
                "event-session-started-durable",
                WORKSPACE_ID,
                "AgentSession",
                &session_id,
                2,
                "agent.session.started.v1",
                serde_json::json!({
                    "agent_session_id": session_id,
                    "task_spec_revision": null,
                    "scope": {"kind": "CONVERSATION", "conversation_id": conversation_id, "conversation_turn_id": turn_id},
                    "agent_binding_id": BINDING_ID,
                    "endpoint_id": ENDPOINT_ID,
                    "runtime_id": RUNTIME_ID,
                    "runtime_incarnation_id": INCARNATION_ID,
                    "session_state": "ACTIVE",
                    "reported_at": NOW,
                }),
            ),
            turn_event: draft(
                "event-turn-running-durable",
                WORKSPACE_ID,
                "ConversationTurn",
                &turn_id,
                2,
                "conversation.turn.status.changed.v1",
                serde_json::json!({
                    "turn_id": turn_id,
                    "from": "OPEN",
                    "to": "RUNNING",
                    "reason_code": "SESSION_READY",
                    "agent_session_id": session_id,
                    "aggregate_version": 2,
                }),
            ),
        })
        .expect("activation succeeds");
    let store = crate::conversations::SqliteConversationStore::new(fixture.store.clone());
    Activated {
        fixture,
        store,
        conversation_id,
        turn_id,
        session_id,
    }
}

impl Activated {
    fn commit(&self, request_id: &str, response_id: &str, text: &str) -> CommitConversationAgentResponse {
        CommitConversationAgentResponse {
            request: WorkspaceCreateRequest {
                principal_id: OWNER_ID.to_owned(),
                request_id: request_id.to_owned(),
                request_payload: serde_json::json!({"answer": text}),
            },
            principal_id: OWNER_ID.to_owned(),
            workspace_id: WORKSPACE_ID.to_owned(),
            conversation_id: self.conversation_id.clone(),
            turn_id: self.turn_id.clone(),
            agent_session_id: self.session_id.clone(),
            expected_turn_version: 2,
            expected_session_version: 2,
            retry_ordinal: 0,
            provider_response_id: response_id.to_owned(),
            turn_status: ConversationTurnStatus::Completed,
            usage: None,
            message: ConversationMessageRecord {
                message_id: format!("message-{request_id}"),
                conversation_id: self.conversation_id.clone(),
                author: serde_json::json!({"kind": "AGENT", "agent_binding_id": BINDING_ID}),
                role: "AGENT".to_owned(),
                agent_session_id: Some(self.session_id.clone()),
                agent_binding_id: Some(BINDING_ID.to_owned()),
                turn_id: Some(self.turn_id.clone()),
                content: serde_json::json!([{"kind": "TEXT", "text": text}]),
                resource_refs: serde_json::json!([]),
                source_channel_ref: None,
                created_at: NOW.to_owned(),
            },
            message_event: draft(
                &format!("event-msg-{request_id}"),
                WORKSPACE_ID,
                "Conversation",
                &self.conversation_id,
                3,
                "conversation.message.added.v1",
                serde_json::json!({"conversation_id": self.conversation_id, "message_id": format!("message-{request_id}"), "role": "AGENT"}),
            ),
            turn_event: draft(
                &format!("event-turn-{request_id}"),
                WORKSPACE_ID,
                "ConversationTurn",
                &self.turn_id,
                3,
                "conversation.turn.settled.v1",
                serde_json::json!({"turn_id": self.turn_id, "from": "RUNNING", "to": "COMPLETED", "aggregate_version": 3}),
            ),
            session_event: draft(
                &format!("event-session-{request_id}"),
                WORKSPACE_ID,
                "AgentSession",
                &self.session_id,
                3,
                "agent.session.closed.v1",
                serde_json::json!({"agent_session_id": self.session_id, "from": "ACTIVE", "to": "CLOSED", "aggregate_version": 3}),
            ),
        }
    }

    fn connection(&self) -> Connection {
        Connection::open(database_path(&self.fixture.directory)).expect("open fixture database")
    }

    fn counts(&self) -> (i64, i64, i64, i64, String, String) {
        self.connection()
            .query_row(
                "SELECT (SELECT COUNT(*) FROM conversation_messages WHERE role='AGENT'),
                        (SELECT COUNT(*) FROM conversation_response_commits),
                        (SELECT COUNT(*) FROM conversation_turn_usage_records),
                        (SELECT COUNT(*) FROM request_dedup WHERE request_id LIKE 'req-%'),
                        (SELECT status FROM conversation_turns WHERE turn_id=?1),
                        (SELECT status FROM agent_sessions WHERE agent_session_id=?2)",
                params![self.turn_id, self.session_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
            )
            .expect("read commit state")
    }
}

#[test]
fn conversation_agent_response_commit_rejects_wrong_author_and_ordinal_without_partial_write() {
    let a = activated();
    let before = a.counts();
    assert_eq!(before, (0, 0, 0, 0, "RUNNING".to_owned(), "ACTIVE".to_owned()));

    let mut wrong_author = a.commit("req-author", "response-1", "hello");
    wrong_author.message.author =
        serde_json::json!({"kind": "AGENT", "agent_binding_id": "some-other-binding"});
    assert!(a.store.commit_conversation_agent_response(wrong_author).is_err());

    let mut user_author = a.commit("req-user-author", "response-1", "hello");
    user_author.message.author = serde_json::json!({"kind": "USER", "principal_id": OWNER_ID});
    assert!(a.store.commit_conversation_agent_response(user_author).is_err());

    let mut wrong_ordinal = a.commit("req-ordinal", "response-1", "hello");
    wrong_ordinal.retry_ordinal = 1;
    assert!(a.store.commit_conversation_agent_response(wrong_ordinal).is_err());

    let mut wrong_workspace = a.commit("req-workspace", "response-1", "hello");
    wrong_workspace.workspace_id = "another-workspace".to_owned();
    assert!(a.store.commit_conversation_agent_response(wrong_workspace).is_err());

    let mut wrong_session = a.commit("req-session", "response-1", "hello");
    wrong_session.agent_session_id = "another-session".to_owned();
    assert!(a.store.commit_conversation_agent_response(wrong_session).is_err());

    assert_eq!(a.counts(), before, "rejected commits write nothing");
}

#[test]
fn conversation_agent_response_commit_identity_replays_and_rejects_conflicts() {
    let a = activated();
    let first = a
        .store
        .commit_conversation_agent_response(a.commit("req-first", "response-1", "hello"))
        .expect("first commit");

    let replay = a
        .store
        .commit_conversation_agent_response(a.commit("req-second", "response-1", "hello"))
        .expect("same identity and content replays the stored receipt");
    assert_eq!(replay, first);

    let different_content = a
        .store
        .commit_conversation_agent_response(a.commit("req-third", "response-1", "changed"));
    assert!(matches!(
        different_content,
        Err(StoreError::Invalid(ref message)) if message == "CONVERSATION_RESPONSE_IDENTITY_CONFLICT"
    ));
    let different_response = a
        .store
        .commit_conversation_agent_response(a.commit("req-fourth", "response-2", "hello"));
    assert!(matches!(
        different_response,
        Err(StoreError::Invalid(ref message)) if message == "CONVERSATION_RESPONSE_IDENTITY_CONFLICT"
    ));
    let (agent_messages, identities, usage, _, turn, session) = a.counts();
    assert_eq!((agent_messages, identities, usage), (1, 1, 1));
    assert_eq!((turn.as_str(), session.as_str()), ("COMPLETED", "CLOSED"));
    let source: String = a
        .connection()
        .query_row(
            "SELECT attribution_source FROM conversation_turn_usage_records WHERE turn_id=?1",
            params![a.turn_id],
            |row| row.get(0),
        )
        .expect("usage record");
    assert_eq!(source, "UNKNOWN");
}

#[test]
fn conversation_agent_response_commit_rolls_back_on_fault_after_each_insert() {
    let a = activated();
    let before = a.counts();
    let faults = [
        ("conversation_messages", "INSERT", ""),
        ("conversation_response_commits", "INSERT", ""),
        ("conversation_turn_usage_records", "INSERT", ""),
        ("conversation_dispatch_intents", "UPDATE", ""),
        ("agent_session_host_bindings", "DELETE", ""),
        ("domain_events", "INSERT", "WHEN NEW.type = 'conversation.message.added.v1'"),
        ("domain_events", "INSERT", "WHEN NEW.type = 'conversation.turn.settled.v1'"),
        ("domain_events", "INSERT", "WHEN NEW.type = 'agent.session.closed.v1'"),
        ("request_dedup", "INSERT", "WHEN NEW.request_id = 'req-fault'"),
    ];
    for (index, (table, operation, condition)) in faults.iter().enumerate() {
        let connection = a.connection();
        connection
            .execute_batch(&format!(
                "CREATE TRIGGER fault_{index} BEFORE {operation} ON {table} {condition} BEGIN SELECT RAISE(ABORT, 'INJECTED_FAULT'); END;"
            ))
            .expect("install fault trigger");
        drop(connection);
        let result = a
            .store
            .commit_conversation_agent_response(a.commit("req-fault", "response-1", "hello"));
        assert!(result.is_err(), "fault on {operation} {table} must fail the commit");
        assert_eq!(a.counts(), before, "fault on {operation} {table} rolls everything back");
        a.connection()
            .execute_batch(&format!("DROP TRIGGER fault_{index};"))
            .expect("remove fault trigger");
    }
    a.store
        .commit_conversation_agent_response(a.commit("req-fault", "response-1", "hello"))
        .expect("commit succeeds once faults are removed");
}

#[test]
fn conversation_agent_response_commit_two_concurrent_commits_have_one_winner() {
    let a = Arc::new(activated());
    let handles: Vec<_> = (0..2)
        .map(|index| {
            let a = Arc::clone(&a);
            std::thread::spawn(move || {
                a.store.commit_conversation_agent_response(a.commit(
                    &format!("req-race-{index}"),
                    &format!("response-{index}"),
                    &format!("answer {index}"),
                ))
            })
        })
        .collect();
    let results: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().expect("commit thread"))
        .collect();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    let (agent_messages, identities, ..) = a.counts();
    assert_eq!((agent_messages, identities), (1, 1));
}

#[test]
fn conversation_dispatch_intent_claim_is_compare_and_swap() {
    let a = activated();
    let unclaimed = a
        .store
        .list_conversation_dispatch_intents(ConversationDispatchState::Unclaimed, 10)
        .expect("list unclaimed");
    assert_eq!(unclaimed.len(), 1);
    assert_eq!(unclaimed[0].turn_id, a.turn_id);
    assert_eq!(unclaimed[0].workspace_id, WORKSPACE_ID);
    assert_eq!(unclaimed[0].retry_ordinal, 0);

    let store = Arc::new(a.store.clone());
    let wins: usize = (0..4)
        .map(|index| {
            let store = Arc::clone(&store);
            let turn_id = a.turn_id.clone();
            std::thread::spawn(move || {
                store
                    .claim_conversation_dispatch_intent(
                        &turn_id,
                        0,
                        &format!("incarnation-{index}"),
                        NOW,
                    )
                    .expect("claim attempt")
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|handle| usize::from(handle.join().expect("claim thread")))
        .sum();
    assert_eq!(wins, 1, "exactly one dispatcher wins the claim");
    assert!(
        a.store
            .list_conversation_dispatch_intents(ConversationDispatchState::Unclaimed, 10)
            .expect("list unclaimed")
            .is_empty()
    );
    let claimed = a
        .store
        .list_conversation_dispatch_intents(ConversationDispatchState::Claimed, 10)
        .expect("list claimed");
    assert_eq!(claimed.len(), 1);
    assert!(claimed[0].claimed_by_runtime_incarnation.is_some());

    assert!(a.store.release_conversation_dispatch_intent(&a.turn_id, 0).expect("release"));
    assert!(!a.store.release_conversation_dispatch_intent(&a.turn_id, 0).expect("release twice"));
    assert!(a.store.claim_conversation_dispatch_intent(&a.turn_id, 0, "incarnation-next", NOW).expect("re-claim"));

    // Settlement through the final-response commit is terminal for the intent.
    a.store
        .commit_conversation_agent_response(a.commit("req-settle", "response-1", "hello"))
        .expect("commit settles");
    assert!(!a.store.claim_conversation_dispatch_intent(&a.turn_id, 0, "incarnation-late", NOW).expect("late claim"));
    assert!(!a.store.release_conversation_dispatch_intent(&a.turn_id, 0).expect("late release"));
    let delete = a
        .connection()
        .execute("DELETE FROM conversation_dispatch_intents", []);
    assert!(delete.is_err(), "intents are never deleted");
}
