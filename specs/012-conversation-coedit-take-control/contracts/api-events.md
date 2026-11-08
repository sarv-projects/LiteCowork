# API and events - Conversation co-edit and take-control

## Existing served operations reused

- `POST /v1/conversations/{conversation_id}/turns` (`submitConversationTurn`) and the Conversation cancel operation are owned by Feature 001. Take-control reuses the cancel path and the turn-settlement authority; it does not introduce a separate relay-style send API.
- `conversation.turn.created.v2` pins the per-turn binding. Co-edit never reroutes an admitted turn.
- The cancel route currently commits `CANCEL_REQUESTED` then `CANCELLED` without observing native interrupt/quiescence (register A-25). Take-control depends on that defect being fixed before it can be qualified.

## Target-only operations (PROPOSED, not served)

These are PROPOSED and promotable only when the canonical owner document and `docs/schemas/` change together in the same reviewed change:

- A take-control command on a non-terminal turn (`If-Match` turn version + `Idempotency-Key`) that issues the Feature 001 cancel path, waits for observed quiescence, and atomically settles the turn `CANCELLED` with an immutable `OWNER_TOOK_CONTROL` control event.
- An owner side-chat message of kind `OWNER_SIDE_NOTE` correlated to a non-terminal turn id, committed as ordinary bounded Conversation history and excluded from that turn's `ConversationContextPackage`.

## Events

Reuse the canonical Feature 001 turn-settlement events. Any new `conversation.control.*` event for the control-event record, or a new message-kind event for `OWNER_SIDE_NOTE`, must be added once in [docs/EVENTS.md](../../../docs/EVENTS.md) with schemas in `docs/schemas/domain-event.schema.json` before implementation. This feature declares no new event in DRAFT.

## Runtime boundary

Take-control issues the Feature 001 cancel path to `AgentSessionSupervisor`; the native stop/quiescence step happens outside any SQLite transaction. Side-chat never enters the native prompt and never starts, steers or reroutes a native session. Do not represent co-edit as provider-capable beyond the one wired OpenCode cancel path.

## Errors and compatibility

Use the existing owner error families for stale Conversation/turn version, idempotency-key reuse with a conflicting payload, and Feature 001 turn settlement failures. New typed errors (`TakeControlError`, `SideNoteError`) are PROPOSED and must be present in [docs/SCHEMAS.md](../../../docs/SCHEMAS.md) and `docs/schemas/error-codes.schema.json` before they are promotable. Do not invent co-edit-specific error codes in this package without owner promotion.
