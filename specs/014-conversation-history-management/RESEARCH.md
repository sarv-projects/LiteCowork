# Research and Decision Log - Conversation History Management

**Feature:** 014 | **Status:** IN_REVIEW - not owner-accepted; does not authorize implementation; not a claim of implementation or qualification.
**Checked:** 2026-11-09 - repository source and docs read; no web research; no external code copied.

## 1. Pinned external sources

No external implementation is adopted.

| Source | Revision | License | Finding | Decision |
|---|---|---|---|---|
| OpenHands `OpenHands/OpenHands` shell, `src/utils/conversation-archive-status.ts` and `src/hooks/query/conversation-history-query-options.ts` (listed in [REFERENCE-REPOS-CORE-ARCHITECTURE-INDEX.md](../../docs/REFERENCE-REPOS-CORE-ARCHITECTURE-INDEX.md)) | `22e3057c89b2e5245a5a36fd9c48e52c8a984ba4` | MIT | Read-only history presentation separates `MISSING` from `ERROR`; history queries are keyed so a changed host refetches | Prototype the idea only (distinct missing/error/deleted states, keyed queries); copy no code |
| ContextDocument purge (this repository) | current checkout | repository license | Sealed manifest, per-target receipts, restart resume | Reuse the pattern (same codebase) |

SQLite FTS5 behaviour (deleted terms remain in segments unless `secure-delete` is enabled) is general SQLite knowledge, not verified against the bundled build here (O-01).

## 2. Repository sources inspected

| Source | Path | Finding |
|---|---|---|
| Routes | `apps/litecoworkd/src/conversation_operator.rs` | No search/rename/archive/delete; title bound 1-160; list cursor `(created_at, conversation_id)` |
| Storage types | `crates/storage-core/src/conversation.rs` | `ConversationRecord` title only; `SetConversationAgentBinding` mutation pattern |
| SQLite | `crates/storage-sqlite/src/conversations.rs` | Single-statement list query; no status |
| Schema | `docs/schemas/sqlite-v1.sql`, `sqlite-v12.sql` | Tables with `conversation_id` enumerated in SPEC section 4; plaintext message content; snapshot blobs |
| Events | `docs/EVENTS.md`; `conversation_operator.rs` payloads | Created event has no title; message event has digest only |
| Purge precedent | `crates/storage-sqlite/src/context_document_purge.rs`; STATE-MACHINES Resource section; API ContextDocument deletion | Sealed manifest, per-target receipts, restart resume |
| Index precedent | `crates/storage-sqlite/src/resource_index.rs` | HMAC term tokens; bounds |
| Shell rules | `docs/DESKTOP-SHELL-LLD.md` left sidebar | Search permission-scoped and capped; missing index differs from no results; no cascade |
| Search scope | `docs/RICH-RESPONSE.md` "Search, export, and channels" | Index semantic content only |
| Coworker chats | `docs/COWORKERS-TARGET.md` | Deletion/archive must separate removing a view from deleting Task artifacts and memories; activity ordering |
| UI | `apps/litecowork-ui/src/App.tsx` Recent Chats; `ConversationsPage.tsx` header | No search; "now" text; inert More options; fabricated fallback title |

## 3. Alternatives

| Alternative | Decision | Rationale |
|---|---|---|
| Immediate hard delete in one transaction | Reject | Blobs and WAL cannot be removed atomically with rows; no receipts; crash leaves orphans |
| Tombstone with immediate fenced purge | Select (B-38 proposal) | Matches the ContextDocument precedent; content unreadable at once; receipts prove removal |
| Tombstone with 30-day restore window | Owner option | Keeps content on disk; Archive already covers reversible removal |
| Plaintext FTS5 index | Flexible, not preferred | Second plaintext copy; segment remnants after delete |
| HMAC term-token index | Preferred (F-01) | Shares tokenizer with Resource index; purge is a row delete |
| Async indexer from event journal | Flexible (F-02 b) | Adds a lag window the UI must explain |
| Cascade delete Tasks and Artifacts | Reject | Violates L-03 |

## 4. Open research

| ID | Topic | Next evidence | Owner |
|---|---|---|---|
| R-01 | FTS5 availability in bundled `libsqlite3-sys 0.38.2` | Build flags or a compile test | Storage |
| R-02 | WAL and free-page residue after purge | Canary scan with and without `secure_delete` pragma and checkpoint | Storage |
| R-03 | Evidence records that quote Conversation text | Feature 005/007 Evidence owner contract | Trust/Evidence |
| R-04 | Search latency on release-row hardware | Seeded benchmark in 014-T15 | Storage |
