# Data model - Conversation co-edit and take-control

**Feature:** 012 | **Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.

Identities and lifecycles stay in the canonical owner documents. This file lists the records this slice reads or commits. It does not authorize new tables by itself.

## Records

- ConversationTurn - reused from Feature 001; take-control settles it `CANCELLED` after observed quiescence. No new turn type.
- ConversationControlEvent - immutable `OWNER_TOOK_CONTROL` / `OWNER_RELEASED_CONTROL` record correlated to the taken-over turn. Accepted target record (not yet in source); promoted with the owner doc and schema together by TASKS T01.
- OwnerSideNote - bounded owner message, kind `OWNER_SIDE_NOTE`, correlated to a non-terminal turn id, excluded from that turn's native prompt. Accepted target record (not yet in source).

## Relations

Workspace is the security boundary. Conversation owns the turn, control events and side-notes; child records carry workspace and conversation id. Runtime-local secrets, handles and private locators are excluded from replicated events and ordinary backups.

## Lifecycle

Take-control: cancel request -> observe native quiescence (outside the transaction) -> atomic settle `CANCELLED` + control-event. A racing completion/failure wins and remains authoritative. Side-note: single bounded append, valid only while the correlated turn is non-terminal. Terminal states are explicit; unknown is not success.

## Persistence and events

Reuse the Feature 001 turn-settlement events. Any new control-event or side-note event requires a same-change edit to [docs/EVENTS.md](../../docs/EVENTS.md) and `docs/schemas/domain-event.schema.json`, and any new message kind requires a same-change edit to the Conversation owner and `docs/schemas/`.

## Retention

Follow the owner retention rules. Purge removes future visibility of side-note payload bytes; the control-event receipt of the take-over may remain as provenance.

## Migration

None in source. B-34 is accepted; the migration is designed by TASKS T05 after the Feature 001 Stop/quiescence fix (A-25) lands, as the next immutable `sqlite-vNN.sql`.

## Current source boundary (2026-11-08)

- There is no co-edit, take-control or side-chat record in the active source. The Conversation turn and a cancel route exist (Feature 001), but cancel does not observe native quiescence and the UI Stop is ineffective for the OpenCode path (register A-25).
- `ConversationControlEvent` and `OwnerSideNote` are accepted target records, not persisted aggregates or tables today. The context-package builder has no side-note exclusion because no side-note exists yet.
- Native session IDs, handles, private harness transcripts and credentials are not part of any co-edit projection and must stay excluded from events, snapshots, API responses and logs.
