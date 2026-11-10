use super::conversation_turn_tests::fixture;
use super::*;
use domain_conversation::{ConversationTurnStatus, TurnCommand};
use storage_core::{
    AppendOwnerSideNote, ConversationCoeditStore, ConversationTurnStore, OwnerControlEventInsert,
    WorkspaceCreateRequest, conversation::SettleConversationTurn,
};

const AT: &str = "2026-10-09T12:10:00Z";

fn side_note(request_id: &str, note_id: &str, turn_id: &str, body: &str) -> AppendOwnerSideNote {
    AppendOwnerSideNote {
        request: WorkspaceCreateRequest {
            principal_id: "owner-local".to_owned(),
            request_id: request_id.to_owned(),
            request_payload: serde_json::json!({
                "operation": "conversation.side_note.append.v1",
                "workspace_id": "workspace-turn",
                "conversation_id": "conversation-turn",
                "correlated_turn_id": turn_id,
                "body": body,
            }),
        },
        principal_id: "owner-local".to_owned(),
        workspace_id: "workspace-turn".to_owned(),
        conversation_id: "conversation-turn".to_owned(),
        correlated_turn_id: turn_id.to_owned(),
        side_note_id: note_id.to_owned(),
        body: body.to_owned(),
        created_at: AT.to_owned(),
    }
}

fn set_turn_status(directory: &tempfile::TempDir, status: &str) {
    let connection = Connection::open(state_database(directory)).expect("open state");
    connection
        .execute(
            "UPDATE conversation_turns SET status=?1 WHERE turn_id='turn-one'",
            [status],
        )
        .expect("force turn status");
}

fn settle(control: Option<OwnerControlEventInsert>) -> SettleConversationTurn {
    SettleConversationTurn {
        principal_id: "owner-local".to_owned(),
        workspace_id: "workspace-turn".to_owned(),
        conversation_id: "conversation-turn".to_owned(),
        turn_id: "turn-one".to_owned(),
        expected_turn_version: 1,
        command: TurnCommand::SettleCancelled {
            settled_at: AT.to_owned(),
        },
        turn_event: EventDraft {
            event_id: "event-take-control-settle".to_owned(),
            workspace_id: "workspace-turn".to_owned(),
            entity_type: "ConversationTurn".to_owned(),
            entity_id: "turn-one".to_owned(),
            origin_runtime_id: "runtime-coedit".to_owned(),
            entity_revision: 2,
            hlc_timestamp: AT.to_owned(),
            correlation_id: "correlation-take-control".to_owned(),
            causation_id: None,
            schema_version: 1,
            event_type: "conversation.turn.settled.v1".to_owned(),
            payload: serde_json::json!({
                "turn_id": "turn-one",
                "from": "CANCEL_REQUESTED",
                "to": "CANCELLED",
                "aggregate_version": 2,
            }),
            recorded_at: AT.to_owned(),
        },
        session: None,
        usage: None,
        control_event: control,
    }
}

fn control_event_count(directory: &tempfile::TempDir) -> i64 {
    Connection::open(state_database(directory))
        .expect("open state")
        .query_row("SELECT COUNT(*) FROM conversation_control_events", [], |row| {
            row.get(0)
        })
        .expect("count control events")
}

#[test]
fn conversation_coedit_side_note_commits_replays_and_stays_out_of_the_prompt() {
    let (_directory, store, turn) = fixture();
    let conversations = SqliteConversationStore::new(store);
    conversations
        .create_conversation_turn(turn)
        .expect("turn admitted");
    let first = conversations
        .append_owner_side_note(side_note("note-one", "note-id-1", "turn-one", "remember X"))
        .expect("side note commits");
    assert_eq!(first.body, "remember X");
    assert_eq!(
        conversations
            .append_owner_side_note(side_note("note-one", "note-id-1", "turn-one", "remember X"))
            .unwrap(),
        first
    );
    assert!(
        conversations
            .append_owner_side_note(side_note("note-one", "note-id-1", "turn-one", "changed"))
            .is_err(),
        "changed payload under one key must not replay"
    );
    let listed = conversations
        .list_owner_side_notes("owner-local", "workspace-turn", "conversation-turn", "turn-one")
        .unwrap();
    assert_eq!(listed, vec![first]);

    // AC-009: a side note never creates a turn or a message, so the native prompt is unchanged.
    let package = conversations
        .load_conversation_context_for_turn(
            "owner-local",
            "workspace-turn",
            "conversation-turn",
            "turn-one",
        )
        .expect("context loads");
    assert!(!format!("{package:?}").contains("remember X"));
}

#[test]
fn conversation_coedit_side_note_refuses_terminal_over_budget_and_foreign_without_writes() {
    let (directory, store, turn) = fixture();
    let conversations = SqliteConversationStore::new(store);
    conversations.create_conversation_turn(turn).unwrap();
    assert_eq!(
        conversations
            .append_owner_side_note(side_note("note-big", "n-big", "turn-one", &"x".repeat(4_001)))
            .unwrap_err(),
        StoreError::Invalid("SIDE_NOTE_OVER_BUDGET".to_owned())
    );
    assert_eq!(
        conversations
            .append_owner_side_note(side_note("note-empty", "n-empty", "turn-one", "  "))
            .unwrap_err(),
        StoreError::Invalid("SIDE_NOTE_OVER_BUDGET".to_owned())
    );
    let mut foreign = side_note("note-foreign", "n-foreign", "turn-one", "hi");
    foreign.principal_id = "someone-else".to_owned();
    foreign.request.principal_id = "someone-else".to_owned();
    assert_eq!(
        conversations.append_owner_side_note(foreign).unwrap_err(),
        StoreError::NotFound
    );
    assert_eq!(
        conversations
            .append_owner_side_note(side_note("note-missing", "n-missing", "no-such-turn", "hi"))
            .unwrap_err(),
        StoreError::NotFound
    );
    set_turn_status(&directory, "FAILED");
    assert_eq!(
        conversations
            .append_owner_side_note(side_note("note-late", "n-late", "turn-one", "too late"))
            .unwrap_err(),
        StoreError::Invalid("SIDE_NOTE_TURN_TERMINAL".to_owned())
    );
    let rows: i64 = Connection::open(state_database(&directory))
        .unwrap()
        .query_row("SELECT COUNT(*) FROM conversation_owner_side_notes", [], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 0);
}

#[test]
fn conversation_coedit_take_control_settles_cancelled_and_appends_control_event_atomically() {
    let (directory, store, turn) = fixture();
    let conversations = SqliteConversationStore::new(store);
    conversations.create_conversation_turn(turn).unwrap();
    set_turn_status(&directory, "CANCEL_REQUESTED");
    let settled = conversations
        .settle_conversation_turn(settle(Some(OwnerControlEventInsert {
            control_event_id: "control-1".to_owned(),
            occurred_at: AT.to_owned(),
        })))
        .expect("settlement plus control event commits");
    assert_eq!(settled.turn.status, ConversationTurnStatus::Cancelled);
    assert_eq!(control_event_count(&directory), 1);
    // The append-only table refuses edits and deletes.
    let connection = Connection::open(state_database(&directory)).unwrap();
    assert!(connection
        .execute("UPDATE conversation_control_events SET kind='OWNER_RELEASED_CONTROL'", [])
        .is_err());
    assert!(connection
        .execute("DELETE FROM conversation_control_events", [])
        .is_err());
}

#[test]
fn conversation_coedit_failed_control_event_insert_rolls_back_the_settlement() {
    let (directory, store, turn) = fixture();
    let conversations = SqliteConversationStore::new(store);
    conversations.create_conversation_turn(turn).unwrap();
    set_turn_status(&directory, "CANCEL_REQUESTED");
    // Reusing an id forces the control-event insert to fail after the turn update.
    let connection = Connection::open(state_database(&directory)).unwrap();
    connection
        .execute(
            "INSERT INTO conversation_control_events(control_event_id,workspace_id,conversation_id,turn_id,kind,occurred_at) VALUES('control-dup','workspace-turn','conversation-turn','turn-one','OWNER_RELEASED_CONTROL',?1)",
            [AT],
        )
        .unwrap();
    drop(connection);
    let result = conversations.settle_conversation_turn(settle(Some(OwnerControlEventInsert {
        control_event_id: "control-dup".to_owned(),
        occurred_at: AT.to_owned(),
    })));
    assert!(result.is_err());
    let status: String = Connection::open(state_database(&directory))
        .unwrap()
        .query_row("SELECT status FROM conversation_turns WHERE turn_id='turn-one'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(status, "CANCEL_REQUESTED", "turn must not be CANCELLED without the control event");
}
