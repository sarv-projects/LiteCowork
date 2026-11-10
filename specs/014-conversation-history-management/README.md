# 014 - Conversation History Management

- **Status:** IN_REVIEW - authored 2026-11-09; not owner-accepted; does not authorize implementation; not a claim of implementation or qualification.
- **Revision:** 0.1 | **Updated:** 2026-11-09 | **Decision:** B-38 (delete semantics: tombstone plus purge window vs immediate purge) ACCEPTED as proposed by the owner (sarv-projects) on 2026-11-10, register A-62, and treated as unblocking implementation; whole-SPEC acceptance and the ACCEPTED/LOCKED flip are still pending, so the package stays IN_REVIEW.
- **Implementation state (2026-11-10):** Storage layer plus the Operator history routes. Storage: T01, T02, T03 PARTIAL - migration v31 (`conversation_history_state` side table) and `ConversationHistoryStore` (rename, archive, status get/list) with tests in `conversation_history_tests.rs`. Routes: T07 PARTIAL - `PATCH /v1/conversations/{id}` (rename), `POST .../archive`, `POST .../unarchive` and the `status` filter on `GET /v1/conversations` at `apps/litecoworkd/src/conversation_operator.rs`, with two route tests in `conversation_operator_tests.rs`. Still absent: the search index and route (T04, T08), tombstone/purge and the delete routes (T05, T06, T09), the Tauri commands (T10) and every UI change (T11, T12). T13 onward NOT STARTED.
- **Owners:** Conversation, Storage, Operator/API, Desktop UI (sidebar and Conversation header), Security/Privacy and Product.

## User outcome

The owner can find a past Conversation by title or by the text of committed messages, rename it, archive and unarchive it, and delete it. Delete makes content unreadable immediately, purges it from every LiteCowork-owned store with receipts, survives crash, restart and restore without resurrecting content, and never silently deletes independent Tasks, Artifacts or memory.

## Current truth

The Operator serves Conversation create, list, get, turn submit/retry/cancel and agent-binding change, plus the 014 history routes: rename, archive, unarchive and a status-filtered list (`apps/litecoworkd/src/conversation_operator.rs`). `ConversationRecord` has a create-time title and no status field: lifecycle status lives in the `conversation_history_state` side table, is filterable through `GET /v1/conversations?status=`, and is not yet returned on a Conversation read. The sidebar "Recent Chats" list in `apps/litecowork-ui/src/App.tsx` has no search or row actions and shows the literal text "now" on the selected row; the Conversation header "More options" button has no handler and the header falls back to the fabricated title "AI Capabilities Overview". A tombstone and purge precedent exists for ContextDocuments (`crates/storage-sqlite/src/context_document_purge.rs`).

## Package

- [SPEC.md](SPEC.md) - problem, decision B-38, HLD/LLD, security, EARS acceptance.
- [PLAN.md](PLAN.md) - phases, dependency graph, gates.
- [TASKS.md](TASKS.md) - 19 ordered tasks to V1 VERIFIED.
- [DATA-MODEL.md](DATA-MODEL.md) - status, search index, tombstone and purge records.
- [RESEARCH.md](RESEARCH.md) - sources, alternatives, open research.
- [WIRING.yaml](WIRING.yaml) - components, edges, transactions, recovery.
- [contracts/](contracts/README.md) - API/event, errors/recovery and UI behavior.

## Canonical owners

[Architecture](../../ARCHITECTURE.md), [V1 release contract](../../docs/V1-PRODUCT-RELEASE-CONTRACT.md), [State Machines](../../docs/STATE-MACHINES.md), [API](../../docs/API.md), [Events](../../docs/EVENTS.md), [Storage](../../docs/STORAGE.md), [Data Model](../../docs/DATA-MODEL.md), [Desktop Shell LLD](../../docs/DESKTOP-SHELL-LLD.md), [Rich Response](../../docs/RICH-RESPONSE.md), [Failure Recovery](../../docs/FAILURE-RECOVERY.md), [Observability](../../docs/OBSERVABILITY.md), [Acceptance and E2E plan](../../docs/ACCEPTANCE-AND-E2E-PLAN.md).
