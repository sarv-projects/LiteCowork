# Data Model - Conversation History Management

**Status:** IN_REVIEW - not owner-accepted; does not authorize implementation; not a claim of implementation or qualification.
**Feature:** 014 | **Authority:** DATA-MODEL.md, STATE-MACHINES.md, STORAGE.md, EVENTS.md, SCHEMAS.md.

This feature extends the canonical Conversation record. It must not create a second Conversation catalog or a second deletion framework. Nothing below exists in source today; the migration chain ends at `sqlite-v28.sql`.

## 1. Existing records

| Record | Source | Feature use |
|---|---|---|
| Conversation | `conversations` (`sqlite-v1.sql`); `ConversationRecord` | Gains status and timestamps |
| ConversationMessage | `conversation_messages` (plaintext `content_json`) | Search source; purge target |
| ConversationTurn | `conversation_turns` | Active-turn check; purge of content-bearing fields |
| UserRequest / UserRequestResponse | `user_requests`, `user_request_responses` | Quiescence check; purge targets |
| RichPresentation | `rich_presentations` (`sqlite-v12.sql`) and document blobs | Fence and purge target |
| AggregateSnapshot | `aggregate_snapshots.blob_ref_json` | Snapshot blobs for Conversation and messages are purge targets |
| Request receipt | `request_dedup.response_json` | Conversation command responses are purge targets |
| DomainEvent | `domain_events` | Retained; message events carry digests only |
| Task | `tasks.conversation_id` | Retained; origin shown as deleted conversation |
| Capability grants/activations/invocations | `capability_*` with `conversation_id` | Quiescence check; Evidence retention per O-02 |

## 2. Proposed fields and records (not yet in source)

| Record | Fields | Notes |
|---|---|---|
| Conversation (extended) | `status` (`ACTIVE`, `ARCHIVED`, `DELETION_PENDING`, `DELETED`), `archived_at`, `deleted_at` | Title cleared at tombstone |
| ConversationSearchTerm | `workspace_id`, `conversation_id`, `message_id` (null for title), `term_token`, `key_version` | F-01 option (a): HMAC tokens, no plaintext |
| ConversationSearchIndexState | `workspace_id`, `state` (`BUILDING`, `READY`), `watermark` | Backfill progress |
| ConversationPurgePlan | `plan_id`, `workspace_id`, `conversation_id`, `sealed_at`, `target_count` | Immutable once sealed |
| ConversationPurgeReceipt | `plan_id`, `target_id`, `target_kind`, `target_ref`, `state` (`PENDING`, `ACKNOWLEDGED`), `acknowledged_at` | One per target |

List index: `(workspace_id, status, created_at DESC, conversation_id DESC)`.

## 3. Lifecycle

```text
ACTIVE <-> ARCHIVED
ACTIVE | ARCHIVED -> DELETION_PENDING -> DELETED
```

- Rename: ACTIVE only; version +1.
- Archive: ACTIVE with no nonterminal turn; unarchive: ARCHIVED -> ACTIVE.
- Delete: ACTIVE or ARCHIVED, quiescent; DELETION_PENDING fences all reads; DELETED after every receipt is acknowledged. DELETED is terminal.

## 4. Purge manifest target kinds (B-38)

`MESSAGE_ROW`, `SEARCH_TERMS`, `TURN_CONTENT`, `USER_REQUEST_DETAIL`, `USER_REQUEST_RESPONSE`, `RICH_PRESENTATION_ROW`, `RICH_PRESENTATION_BLOB`, `SNAPSHOT_BLOB`, `REQUEST_RECEIPT_RESPONSE`, `CHANNEL_THREAD_MAPPING`. Bounded like the ContextDocument manifest; an oversized manifest refuses delete with no partial write.

## 5. Events (proposed)

| Event | Payload |
|---|---|
| `conversation.renamed.v1` | conversation_id, aggregate_version, title_digest (or null) |
| `conversation.archived.v1` | conversation_id, aggregate_version |
| `conversation.unarchived.v1` | conversation_id, aggregate_version |
| `conversation.deletion_requested.v1` | conversation_id, aggregate_version, plan_id, target_count |
| `conversation.deleted.v1` | conversation_id, plan_id |

## 6. Retention, backup and migration

- Tombstone and events retained for the Workspace lifetime; no title or content in either.
- Feature 017 backups include the tombstone, sealed plan and receipts; a restore resumes purge.
- Content inside older backups is outside the purge guarantee and the delete dialog says so.
- Forward-only migration; existing rows become `ACTIVE`; schema fingerprint updated.
