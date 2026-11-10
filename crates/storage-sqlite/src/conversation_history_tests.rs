use super::conversation_turn_tests::fixture;
use super::*;
use storage_core::{
    ConversationHistoryStore, ConversationStatus, ConversationStore, ConversationTurnStore,
    RenameConversation, SetConversationAgentBinding, SetConversationArchived,
    WorkspaceCreateRequest,
};

const AT: &str = "2026-10-09T12:05:00Z";

fn event(id: &str, event_type: &str, revision: u64) -> EventDraft {
    EventDraft {
        event_id: id.to_owned(),
        workspace_id: "workspace-turn".to_owned(),
        entity_type: "Conversation".to_owned(),
        entity_id: "conversation-turn".to_owned(),
        origin_runtime_id: "runtime-history".to_owned(),
        entity_revision: revision,
        hlc_timestamp: AT.to_owned(),
        correlation_id: format!("correlation-{id}"),
        causation_id: None,
        schema_version: 1,
        event_type: event_type.to_owned(),
        payload: serde_json::json!({}),
        recorded_at: AT.to_owned(),
    }
}

fn rename(request_id: &str, title: &str, expected_version: u64) -> RenameConversation {
    RenameConversation {
        request: WorkspaceCreateRequest {
            principal_id: "owner-local".to_owned(),
            request_id: request_id.to_owned(),
            request_payload: serde_json::json!({
                "operation": "conversation.rename.v1",
                "workspace_id": "workspace-turn",
                "conversation_id": "conversation-turn",
                "expected_version": expected_version,
                "title": title,
            }),
        },
        principal_id: "owner-local".to_owned(),
        workspace_id: "workspace-turn".to_owned(),
        conversation_id: "conversation-turn".to_owned(),
        expected_version,
        title: title.to_owned(),
        event: event(
            &format!("event-{request_id}"),
            "conversation.renamed.v1",
            expected_version + 1,
        ),
    }
}

fn archive(request_id: &str, archived: bool, expected_version: u64) -> SetConversationArchived {
    SetConversationArchived {
        request: WorkspaceCreateRequest {
            principal_id: "owner-local".to_owned(),
            request_id: request_id.to_owned(),
            request_payload: serde_json::json!({
                "operation": if archived { "conversation.archive.v1" } else { "conversation.unarchive.v1" },
                "workspace_id": "workspace-turn",
                "conversation_id": "conversation-turn",
                "expected_version": expected_version,
            }),
        },
        principal_id: "owner-local".to_owned(),
        workspace_id: "workspace-turn".to_owned(),
        conversation_id: "conversation-turn".to_owned(),
        expected_version,
        archived,
        event: event(
            &format!("event-{request_id}"),
            if archived {
                "conversation.archived.v1"
            } else {
                "conversation.unarchived.v1"
            },
            expected_version + 1,
        ),
    }
}

#[test]
fn conversation_history_rename_commits_title_event_without_title_and_replays() {
    let (_directory, store, _turn) = fixture();
    let history = SqliteConversationStore::new(store);
    let committed = history
        .rename_conversation(rename("rename-one", "Quarterly plan", 1))
        .expect("rename commits");
    assert_eq!(
        committed.conversation.title.as_deref(),
        Some("Quarterly plan")
    );
    assert_eq!(committed.conversation.version, 2);
    let payload = committed.event.payload.to_string();
    assert!(
        !payload.contains("Quarterly plan"),
        "event must not carry the title"
    );
    assert!(payload.contains("title_digest"));

    let replay = history
        .rename_conversation(rename("rename-one", "Quarterly plan", 1))
        .expect("exact replay");
    assert_eq!(replay, committed);

    let conflict = history
        .rename_conversation(rename("rename-one", "Other title", 1))
        .expect_err("changed payload must conflict");
    assert!(!matches!(conflict, StoreError::NotFound));
    let stored = history
        .get_conversation("owner-local", "workspace-turn", "conversation-turn")
        .unwrap()
        .unwrap();
    assert_eq!(stored.title.as_deref(), Some("Quarterly plan"));
    assert_eq!(stored.version, 2);
}

#[test]
fn conversation_history_rename_rejects_stale_invalid_and_foreign_without_writes() {
    let (_directory, store, _turn) = fixture();
    let history = SqliteConversationStore::new(store);
    assert!(matches!(
        history.rename_conversation(rename("rename-stale", "Title", 7)),
        Err(StoreError::Conflict {
            expected: Some(7),
            actual: Some(1)
        })
    ));
    assert!(matches!(
        history.rename_conversation(rename("rename-empty", "   ", 1)),
        Err(StoreError::Invalid(_))
    ));
    assert!(matches!(
        history.rename_conversation(rename("rename-long", &"x".repeat(161), 1)),
        Err(StoreError::Invalid(_))
    ));
    let mut foreign = rename("rename-foreign", "Title", 1);
    foreign.principal_id = "someone-else".to_owned();
    foreign.request.principal_id = "someone-else".to_owned();
    assert_eq!(
        history.rename_conversation(foreign).unwrap_err(),
        StoreError::NotFound
    );
    let stored = history
        .get_conversation("owner-local", "workspace-turn", "conversation-turn")
        .unwrap()
        .unwrap();
    assert_eq!(stored.version, 1);
    assert_eq!(stored.title, None);
}

#[test]
fn conversation_history_two_renames_from_one_version_one_wins() {
    let (_directory, store, _turn) = fixture();
    let history = SqliteConversationStore::new(store);
    history
        .rename_conversation(rename("rename-a", "A", 1))
        .expect("first wins");
    assert!(matches!(
        history.rename_conversation(rename("rename-b", "B", 1)),
        Err(StoreError::Conflict {
            expected: Some(1),
            actual: Some(2)
        })
    ));
}

#[test]
fn conversation_history_archive_blocks_turns_and_binding_and_unarchive_restores() {
    let (_directory, store, turn) = fixture();
    let history = SqliteConversationStore::new(store);
    let archived = history
        .set_conversation_archived(archive("archive-one", true, 1))
        .expect("archive an idle Conversation");
    assert_eq!(archived.conversation.version, 2);
    assert_eq!(
        history
            .get_conversation_status("owner-local", "workspace-turn", "conversation-turn")
            .unwrap(),
        Some(ConversationStatus::Archived)
    );
    // Replay returns the stored result.
    assert_eq!(
        history
            .set_conversation_archived(archive("archive-one", true, 1))
            .unwrap(),
        archived
    );

    let active_list = history
        .list_conversations_by_status(
            "owner-local",
            "workspace-turn",
            ConversationStatus::Active,
            None,
            None,
            50,
        )
        .unwrap();
    assert!(active_list.is_empty());
    let archived_list = history
        .list_conversations_by_status(
            "owner-local",
            "workspace-turn",
            ConversationStatus::Archived,
            None,
            None,
            50,
        )
        .unwrap();
    assert_eq!(archived_list.len(), 1);

    // Archived Conversations refuse submit, binding change and rename.
    assert_eq!(
        history.create_conversation_turn(turn).unwrap_err(),
        StoreError::Invalid("CONVERSATION_ARCHIVED".to_owned())
    );
    assert_eq!(
        history
            .rename_conversation(rename("rename-archived", "Nope", 2))
            .unwrap_err(),
        StoreError::Invalid("CONVERSATION_ARCHIVED".to_owned())
    );
    let binding = SetConversationAgentBinding {
        request: WorkspaceCreateRequest {
            principal_id: "owner-local".to_owned(),
            request_id: "binding-archived".to_owned(),
            request_payload: serde_json::json!({
                "operation": "conversation.agent_binding.set.v1",
                "workspace_id": "workspace-turn",
                "conversation_id": "conversation-turn",
                "expected_version": 2,
                "agent_binding_id": null,
            }),
        },
        principal_id: "owner-local".to_owned(),
        workspace_id: "workspace-turn".to_owned(),
        conversation_id: "conversation-turn".to_owned(),
        expected_version: 2,
        agent_binding_id: None,
        event: event(
            "event-binding-archived",
            "conversation.agent_binding.changed.v1",
            3,
        ),
    };
    assert_eq!(
        history
            .set_conversation_agent_binding_idempotent(binding)
            .unwrap_err(),
        StoreError::Invalid("CONVERSATION_ARCHIVED".to_owned())
    );

    let restored = history
        .set_conversation_archived(archive("unarchive-one", false, 2))
        .expect("unarchive");
    assert_eq!(restored.conversation.version, 3);
    assert_eq!(
        history
            .get_conversation_status("owner-local", "workspace-turn", "conversation-turn")
            .unwrap(),
        Some(ConversationStatus::Active)
    );
    assert_eq!(
        history
            .list_conversations_by_status(
                "owner-local",
                "workspace-turn",
                ConversationStatus::Active,
                None,
                None,
                50,
            )
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn conversation_history_archive_refused_while_a_turn_is_nonterminal() {
    let (_directory, store, turn) = fixture();
    let history = SqliteConversationStore::new(store);
    history
        .create_conversation_turn(turn)
        .expect("turn is admitted");
    let version = history
        .get_conversation("owner-local", "workspace-turn", "conversation-turn")
        .unwrap()
        .unwrap()
        .version;
    assert_eq!(
        history
            .set_conversation_archived(archive("archive-busy", true, version))
            .unwrap_err(),
        StoreError::Invalid("CONVERSATION_NOT_QUIESCENT".to_owned())
    );
    assert_eq!(
        history
            .get_conversation_status("owner-local", "workspace-turn", "conversation-turn")
            .unwrap(),
        Some(ConversationStatus::Active)
    );
}
