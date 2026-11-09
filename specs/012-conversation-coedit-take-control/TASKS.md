# Tasks - Conversation co-edit and take-control

**Feature:** 012 | **Status:** ACCEPTED/LOCKED - implementation authorized; tasks not started unless marked.
**Spec revision:** 1.0 | **Release target:** V1 (release matrix rows: Windows x64 + OpenCode v2.0.32 first; Linux x64 second)
**Sequencing (B-34 binding caveat):** every implementation task (T02-T09) waits on Feature 001 task T07 (effective Stop with observed quiescence) and on Feature 001 storage single-flight (Feature 001 task T11). Wave 1, after the 001 Stop fix.

## Current baseline

Verified against the checkout on branch `docs/audit-competitor-superset` (after commit 8129420). Nothing co-edit specific exists in source.

- `apps/litecoworkd/src/conversation_operator.rs` `cancel_conversation_turn` is the only stop path: on RUNNING it commits `TurnCommand::RequestCancel` then immediately `TurnCommand::SettleCancelled` with no native interrupt; on OPEN it settles FAILED with `USER_CANCELLED`. `crates/domain-conversation/src/lib.rs` already models CANCEL_REQUESTED and keeps a racing completion (test `cancel_requires_provider_settlement_and_racing_completion_is_preserved`), but the Operator does not use that race window.
- `apps/litecoworkd/src/agents/opencode_server.rs` has `abort_session` (`/api/session/{session_id}/interrupt`) with no caller; `apps/litecoworkd/src/agents/supervisor.rs` `AgentSessionSupervisor` has no cancellation handle.
- `crates/storage-core/src/conversation.rs` `build_conversation_context_package` maps roles USER/AGENT/CHANNEL, drops `SYSTEM_NOTICE`, and rejects any other role with "unsupported message role". A new side-note role therefore needs an explicit exclusion arm, not a default.
- `ConversationMessageRecord.role` is a free `String`; there is no control-event table, no side-note record and no migration after `docs/schemas/sqlite-v28.sql`.
- Tauri `apps/litecowork-ui/src-tauri/src/conversation_bridge.rs` exposes `cancel_conversation_turn` with `If-Match`; the UI `apps/litecowork-ui/src/conversations/ConversationsPage.tsx` `cancelTurn` targets transient turns only and sends `expectedVersion: 1`; `conversation-view.ts` `active_turn` has no version. There is no driver indicator, take-control control or side-chat composer.
- Tests: none for this package. `apps/litecoworkd/tests/` contains only `cli.rs`.

## 012-T01 - Record owner acceptance and promote the accepted co-edit contracts

- **Status:** PARTIAL - B-34 owner acceptance (scoped) recorded 2026-11-09 in this revision with the RV-14 no-vendor-claim constraint; no interface is promoted into owner docs or docs/schemas.
- **Depends on:** NONE
- **Cross-package prerequisites:** Feature 001 contract promotion task (its T01) lands first so the shared migration number and error registry order are fixed.
- **Owner:** Conversation, Agent Runtime, Product; Events and API owners for promotion.
- **Allowed paths:** `docs/STATE-MACHINES.md`, `docs/EXPERIENCE.md`, `docs/API.md`, `docs/EVENTS.md`, `docs/SCHEMAS.md`, `docs/schemas/operator-api.openapi.yaml`, `docs/schemas/domain-event.schema.json`, `docs/schemas/error-codes.schema.json`, this package.
- **Contract:** SPEC sections 5, 7.2, 10; AC-002, AC-003; SPECIFICATION-STANDARD section 7.
- **Change intent:** promote, labelled "accepted target interface (not yet in source)": the take-control command (exact route path decided here as a sibling of the existing `.../turns/{turn_id}/cancel`, with `If-Match` turn version and `Idempotency-Key`); the side-note command; `ConversationControlEventKind` (`OWNER_TOOK_CONTROL`, `OWNER_RELEASED_CONTROL`) and its event name under `conversation.control.*`; the `OWNER_SIDE_NOTE` message role with `correlated_turn_id`; `TakeControlError` and `SideNoteError` codes; finite side-note byte limit and quiescence deadline values (SPEC section 10 says the exact values are an owner decision; record them here).
- **Acceptance:**
  - [x] B-34 accepted (scoped) and recorded in SPEC section 5.
  - [ ] Each promoted name exists once in its owner doc and machine schema.
  - [ ] Side-note limit and quiescence deadline are numeric and finite.
  - [ ] Negative: schema rejects a side-note without `correlated_turn_id` and a control event without `turn_id`.
- **Command:** `python3 .specify/scripts/validate_feature_package.py` and `python3 scripts/validate_architecture.py`
- **Evidence:** PR link and command output in Completion evidence.
- **Stop:** a promoted name conflicts with a Feature 001 definition; resolve at the owner.

## 012-T05 - Add control-event and side-note storage with CAS and idempotency

- **Status:** BLOCKED - waits on Feature 001 effective Stop/quiescence (B-34 caveat; owner Agent Runtime); no record or table exists.
- **Depends on:** T01
- **Cross-package prerequisites:** Feature 001 single-flight storage task (T11 in that package) and its migration.
- **Owner:** Storage, Conversation.
- **Allowed paths:** `docs/schemas/sqlite-v30.sql` (new; take the next free number after Feature 001 migrations), `crates/storage-core/src/conversation.rs`, `crates/storage-core/src/lib.rs`, `crates/storage-sqlite/src/conversations.rs`, `crates/storage-sqlite/src/lib.rs`, `crates/storage-sqlite/src/conversation_coedit_tests.rs` (new).
- **Contract:** SPEC 7.2, DATA-MODEL; AC-002, AC-003, AC-005, AC-006, AC-009.
- **Change intent:** add `conversation_control_events` (append-only, workspace and conversation scoped, turn id, kind, occurred_at); a store method that settles the turn CANCELLED (`TurnCommand::SettleCancelled`) and appends `OWNER_TOOK_CONTROL` in one IMMEDIATE transaction with receipt; a side-note insert that checks the correlated turn is nonterminal and the body is within the limit inside the same transaction; neither path creates a turn.
- **Acceptance:**
  - [ ] Settlement and control event commit together or not at all (AC-002).
  - [ ] Side-note against a terminal turn fails with `TurnTerminal`; over-limit fails with `OverBudget`; no write (AC-005, AC-006).
  - [ ] Cross-Workspace correlation fails with `CrossWorkspace`.
  - [ ] Neither path inserts a second nonterminal turn (AC-009).
  - [ ] Exact replay returns the receipt; changed payload under the same key conflicts.
- **Command:** `cargo test -p storage-sqlite --locked conversation_coedit` (filter to be added by this task)
- **Evidence:** test output; migration checksum.
- **Stop:** the migration would edit an existing released migration.

## 012-T02 - Implement take-control over a running turn

- **Status:** BLOCKED - waits on Feature 001 effective Stop with observed quiescence (A-25; owner Agent Runtime).
- **Depends on:** T01, T05
- **Cross-package prerequisites:** Feature 001 task T07 (effective Stop and quiescence).
- **Owner:** Agent Runtime, Conversation.
- **Allowed paths:** `apps/litecoworkd/src/conversation_operator.rs`, `apps/litecoworkd/src/conversation_operator_tests.rs`, `apps/litecoworkd/src/agents/supervisor.rs`, `crates/storage-sqlite/src/conversations.rs`.
- **Contract:** SPEC 6, 7.2, 8; AC-001, AC-002, AC-004, AC-008, AC-010, AC-013.
- **Change intent:** add a `take_control_conversation_turn` handler that authenticates, checks owner/Workspace and the `If-Match` turn version, commits CANCEL_REQUESTED through the Feature 001 cancel path, waits for the supervisor's observed quiescence outside any transaction with the T01 deadline, then calls the T05 settle-plus-control-event method. A winning COMPLETED/FAILED is returned as `RacedCompletionWins` with that outcome.
- **Acceptance:**
  - [ ] CANCELLED is settled only after observed quiescence (AC-001).
  - [ ] Control event and settlement are atomic and the stopped turn's committed content is kept (AC-002).
  - [ ] Terminal turn returns `TurnAlreadyTerminal` with no event (AC-004).
  - [ ] Racing completion stays authoritative (AC-010).
  - [ ] Deadline exceeded returns `QuiescenceTimeout` and the turn is not falsely CANCELLED (AC-013).
  - [ ] Responses, events and logs contain no native handle (AC-008).
- **Command:** `cargo test -p litecoworkd --locked conversation_take_control` (filter to be added by this task)
- **Evidence:** test output in Completion evidence.
- **Stop:** the Feature 001 Stop path cannot observe quiescence for the route.

## 012-T03 - Implement correlated side-chat and exclude it from the native prompt

- **Status:** BLOCKED - waits on Feature 001 effective Stop/quiescence (B-34 caveat; owner Agent Runtime).
- **Depends on:** T01, T05
- **Cross-package prerequisites:** NONE beyond the package-level sequencing above.
- **Owner:** Conversation.
- **Allowed paths:** `apps/litecoworkd/src/conversation_operator.rs`, `apps/litecoworkd/src/conversation_operator_tests.rs`, `crates/storage-core/src/conversation.rs`, `crates/storage-sqlite/src/conversations.rs`, `apps/litecoworkd/src/agents/conversation_prompt.rs`.
- **Contract:** SPEC 7.2, 8; AC-003, AC-005, AC-006, AC-007, AC-009.
- **Change intent:** add a side-note handler using the T05 insert; add an explicit `OWNER_SIDE_NOTE` arm in `build_conversation_context_package` that drops the message (like `SYSTEM_NOTICE`) and a guard in `encode_conversation_context_prompt` that refuses one if it ever arrives.
- **Acceptance:**
  - [ ] Side-note commits correlated to the RUNNING turn and does not change the turn version (AC-003).
  - [ ] Terminal-turn and over-limit notes are rejected (AC-005, AC-006).
  - [ ] Context package and the serialized OpenCode prompt contain no side-note text (AC-007).
  - [ ] No second active turn is admitted (AC-009).
- **Command:** `cargo test -p litecoworkd --locked conversation_side_chat` (filter to be added by this task) and `cargo test -p storage-core --locked context`
- **Evidence:** test output in Completion evidence.
- **Stop:** exclusion cannot be proven at the prompt boundary.

## 012-T04 - Project and render the truthful driver indicator

- **Status:** BLOCKED - depends on T02 and T03 (and therefore on Feature 001 Stop/quiescence).
- **Depends on:** T02-T03
- **Cross-package prerequisites:** Feature 001 projected active-turn version (its T16).
- **Owner:** Presentation Operator, Desktop UI.
- **Allowed paths:** `apps/litecoworkd/src/presentation_operator.rs`, `apps/litecowork-ui/src/conversations/conversation-view.ts`, `apps/litecowork-ui/src/conversations/ConversationsPage.tsx`, `apps/litecowork-ui/src/conversations/conversation-driver.ts` (new), `apps/litecowork-ui/tests/conversation-driver.test.ts` (new).
- **Contract:** contracts/ui-behavior.md; AC-012.
- **Change intent:** add `driver` (AGENT_RUNNING, OWNER_IN_CONTROL) and latest control event to the Conversation snapshot; render side-notes with their correlated turn; derive the indicator in a pure TypeScript module from committed status plus latest control event only.
- **Acceptance:**
  - [ ] Indicator is derived from committed status and the latest control event, never from the selected agent (AC-012).
  - [ ] Indicator has an accessible label and live-region announcement on change.
  - [ ] Reload shows the same indicator.
- **Command:** `pnpm --dir apps/litecowork-ui test` and `cargo test -p litecoworkd --locked conversation_snapshot`
- **Evidence:** test output in Completion evidence.
- **Stop:** snapshot cannot carry control state atomically with turn status.

## 012-T06 - Add Tauri commands and the take-control and side-chat composer controls

- **Status:** BLOCKED - depends on T02-T04.
- **Depends on:** T02-T04
- **Cross-package prerequisites:** Feature 001 composer fencing (its T18).
- **Owner:** Desktop bridge, Desktop UI.
- **Allowed paths:** `apps/litecowork-ui/src-tauri/src/conversation_bridge.rs`, `apps/litecowork-ui/src-tauri/src/lib.rs` (command registration only), `apps/litecowork-ui/src/conversations/ConversationsPage.tsx`, `apps/litecowork-ui/src/conversations/conversation-driver.ts`, `apps/litecowork-ui/tests/conversation-driver.test.ts`.
- **Contract:** contracts/ui-behavior.md; AC-001, AC-003, AC-005, AC-010, AC-012.
- **Change intent:** add `take_control_conversation_turn` and `post_conversation_side_note` bridge commands mirroring `cancel_conversation_turn` (If-Match, idempotency key); show "Take control" and a side-chat input only while the projected turn is nonterminal; show "Stopping" until settlement; on `RacedCompletionWins` show the real outcome; preserve the draft on rejection.
- **Acceptance:**
  - [ ] Controls appear only for nonterminal turns and are keyboard operable with accessible names (AC-012).
  - [ ] No "cancelled" text appears before settlement (AC-001); race shows the real outcome (AC-010).
  - [ ] Rejected side-note keeps the draft and shows the exact reason (AC-005).
  - [ ] Offline Operator disables both controls with a reason.
- **Command:** `pnpm --dir apps/litecowork-ui test` and `cargo test -p litecowork-ui --locked conversation_bridge` (filter to be added by this task)
- **Evidence:** test output in Completion evidence.
- **Stop:** NONE beyond test failure.

## 012-T07 - Prove crash, restart, replay and timeout recovery

- **Status:** BLOCKED - depends on T02 and T03.
- **Depends on:** T02-T03
- **Cross-package prerequisites:** Feature 001 restart and orphan recovery sweep (its T08).
- **Owner:** Conversation, Agent Runtime.
- **Allowed paths:** `apps/litecoworkd/src/conversation_operator_tests.rs`, `apps/litecoworkd/src/runtime_recovery_tests.rs`, `crates/storage-sqlite/src/conversation_coedit_tests.rs`.
- **Contract:** contracts/errors-recovery.md; AC-010, AC-011, AC-013.
- **Change intent:** fault-inject daemon stop after CANCEL_REQUESTED and before quiescence, after quiescence and before settlement, and after settlement; verify the Feature 001 sweep reconciles and that an `OWNER_TOOK_CONTROL` event reloads.
- **Acceptance:**
  - [ ] Crash before settlement never shows owner-in-control; the turn follows Feature 001 restart rules.
  - [ ] Owner-in-control and preserved content reload from committed records (AC-011).
  - [ ] Duplicate take-control key replays; conflicting payload fails.
  - [ ] Timeout path stays bounded (AC-013); racing completion stays authoritative (AC-010).
- **Command:** `cargo test -p litecoworkd --locked conversation_coedit_recovery` (filter to be added by this task)
- **Evidence:** test output in Completion evidence.
- **Stop:** any window shows a fabricated CANCELLED.

## 012-T08 - Record the cross-layer trace for take-control and side-chat

- **Status:** NOT STARTED.
- **Depends on:** T06-T07
- **Cross-package prerequisites:** NONE
- **Owner:** Conversation, Desktop UI.
- **Allowed paths:** `specs/012-conversation-coedit-take-control/contracts/ui-behavior.md`, `specs/012-conversation-coedit-take-control/WIRING.yaml`.
- **Contract:** AGENTS.md section 4; AC-001 to AC-013.
- **Change intent:** for Take control and Post side-note record: outcome, control in `ConversationsPage.tsx`, Tauri command, Operator route, authorization, service owner, transaction/version/fence, aggregate/event/receipt, projection, restart behavior and evidence test.
- **Acceptance:**
  - [ ] Both operations have complete rows; WIRING edges are marked served only with T09 evidence.
- **Command:** `python3 .specify/scripts/validate_feature_package.py`
- **Evidence:** updated contracts/ui-behavior.md.
- **Stop:** a hop is missing; add a task instead of documenting it as present.

## 012-T09 - Qualify co-edit on the release matrix

- **Status:** NOT STARTED.
- **Depends on:** T01-T08
- **Cross-package prerequisites:** Feature 001 route qualification (its T09) on the same row; Feature 019 evidence bundle process.
- **Owner:** Product, Conversation, Agent Runtime, Desktop UI.
- **Allowed paths:** `apps/litecoworkd/tests/conversation_coedit_take_control.rs` (new), `apps/litecowork-ui/tests/` (new E2E driver files), this package.
- **Contract:** SPEC sections 12-13; AC-001 to AC-013; docs/ACCEPTANCE-AND-E2E-PLAN.md E2E-01 (stop path). No dedicated E2E id exists for co-edit; this task adds the take-control and side-chat steps to the E2E-01 run and records that mapping.
- **Change intent:** create the canonical target with daemon + SQLite + adapter fixture; run the packaged desktop journey (send, take control during RUNNING, continue by hand, side-note during RUNNING, reload, restart) on Windows x64 + OpenCode v2.0.32, then Linux x64.
- **Acceptance:**
  - [ ] `cargo test -p litecoworkd --test conversation_coedit_take_control --locked` exists and passes.
  - [ ] CODE, SYSTEM and owner USER evidence on Windows x64 includes denial (foreign owner), race, timeout, crash and restart.
  - [ ] Linux x64 row recorded separately; WSL2 not accepted.
- **Command:** `cargo test -p litecoworkd --test conversation_coedit_take_control --locked -- --nocapture` and `pnpm --dir apps/litecowork-ui test`
- **Evidence:** dated evidence bundle per row referenced from IMPLEMENTATION-STATUS.md.
- **Stop:** the Feature 001 route is not qualified on the row.

## 012-T10 - Synchronize status documents and inventories

- **Status:** NOT STARTED.
- **Depends on:** T09
- **Cross-package prerequisites:** NONE (orchestrator owns shared ledgers; this task supplies the edits).
- **Owner:** Conversation, Documentation owners.
- **Allowed paths:** `docs/IMPLEMENTATION-STATUS.md`, `docs/schemas/served-route-inventory.json`, `docs/CONTRADICTION-AND-ALIGNMENT-REGISTER.md`, `docs/V1-UI-ACTION-STATE-LEDGER.md`, `docs/VIABILITY-VS-COMPETITORS.md` (RV-14 status only), this package.
- **Contract:** AGENTS.md sections 4-5; SPEC section 4.
- **Change intent:** regenerate the served-route inventory, add the two new routes with their evidence class, close the B-34 register row, update RV-14 to "mapped to Conversation UX, no vendor claim", refresh SPEC section 4.
- **Acceptance:**
  - [ ] `python3 scripts/audit_served_routes.py` matches the committed inventory.
  - [ ] No QUALIFIED claim without the T09 bundle for that row.
- **Command:** `python3 scripts/audit_served_routes.py` and `python3 .specify/scripts/validate_feature_package.py`
- **Evidence:** diff of status documents.
- **Stop:** claim and evidence disagree; downgrade the claim.

## 012-T11 - Package VERIFIED for V1

- **Status:** NOT STARTED.
- **Depends on:** T01-T10
- **Cross-package prerequisites:** Feature 019 release qualification matrix.
- **Owner:** Product (sign-off), Conversation, Agent Runtime.
- **Allowed paths:** this package; `docs/IMPLEMENTATION-STATUS.md`.
- **Contract:** SPEC sections 12-13; AC-001 to AC-013.
- **Change intent:** assemble the evidence bundle, move ACCEPTED/LOCKED to IMPLEMENTING at the first implementation merge and to VERIFIED after owner sign-off.
- **Acceptance:**
  - [ ] Canonical target passes on both rows with owner USER sign-off.
  - [ ] SPEC and WIRING status equal VERIFIED and the validator passes.
- **Command:** `cargo test -p litecoworkd --test conversation_coedit_take_control --locked -- --nocapture` and `python3 .specify/scripts/validate_feature_package.py`
- **Evidence:** evidence bundle path referenced from IMPLEMENTATION-STATUS.md.
- **Stop:** any row lacks USER evidence.

## Implementation and optimization notes (Rust and TypeScript)

- Take-control should reuse the Feature 001 per-turn cancellation handle rather than adding a second interrupt path; one `abort_session` call site in the adapter keeps quiescence logic in one place (Rust).
- Put the side-note exclusion in `build_conversation_context_package` (Rust, storage-core) so every adapter inherits it; a UI-side filter would not protect the prompt.
- Keep the driver derivation as a pure TypeScript function next to the Feature 001 `conversation-turn-state.ts` module so both share one status model and the existing `node --test` runner covers them.

## Dependency order

T01 -> T05 -> (T02, T03 in parallel) -> T04 -> T06. T07 after T02 and T03. T08 after T06 and T07. T09 after T01-T08 -> T10 -> T11. Nothing after T01 starts before Feature 001 Stop/quiescence lands.

## Cross-package dependencies

| This task | Other package task | Reason |
|---|---|---|
| T01 | Feature 001 contract promotion (T01 there) | Shared migration numbering and error registry |
| T02 and all of T02-T09 | Feature 001 effective Stop with observed quiescence (T07 there) | B-34 caveat: take-control is built on that cancel path |
| T05 | Feature 001 storage single-flight (T11 there) | Co-edit must not open a second active turn |
| T04 | Feature 001 active-turn version in the projection (T16 there) | Take-control needs the real turn version for If-Match |
| T06 | Feature 001 composer fencing (T18 there) | Same composer and status model |
| T07 | Feature 001 restart recovery sweep (T08 there) | Crash reconciliation reuses that sweep |
| T09 | Feature 001 route qualification (T09 there); Feature 019 evidence bundle | Co-edit qualifies only on a qualified Send route |

Feature 001 task numbers refer to `specs/001-conversation-session-supervisor/TASKS.md` revision 1.1.

## Completion evidence

For each task append a row when it closes: task, code revision (SHA), exact command, result, OS and build, harness and version (OpenCode v2.0.32), auth route, evidence class reached (CODE, SYSTEM, USER), report path and reviewer.

| Task | Revision | Command | Result | OS/build | Harness/version | Class | Reviewer |
|---|---|---|---|---|---|---|---|
| T01 (acceptance part) | this revision | `python3 .specify/scripts/validate_feature_package.py` | see report | n/a | n/a | design acceptance only | repository owner |
