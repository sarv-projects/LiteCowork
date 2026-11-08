# Tasks - Conversation co-edit and take-control

**Feature:** 012 | **Status:** DRAFT - do not execute before SPEC is ACCEPTED/LOCKED.

**Spec revision:** 0.1

## 012-T01 - Lock the co-edit owner model

- **Status:** BLOCKED
- **Depends on:** NONE
- **Owner:** Conversation, Agent Runtime and Product
- **Allowed paths:** this package
- **Contract:** SPEC §5
- **Change intent:** Resolve B-34 and confirm the RV-14 no-vendor-claim constraint.
- **Acceptance:** Conversation, Agent Runtime and Product owners approve B-34's two Conversation affordances (take-control over the Feature 001 cancel/quiescence path and the `OWNER_SIDE_NOTE` correlated message), the no-new-durable-Task-type invariant, and the exact quiescence/side-note/idempotency bounds; Product confirms RV-14 Claude Duet is cited as RESEARCH only until a dated first-party page is pinned. No message kind, event or control-event type is inferred from implementation guesswork.
- **Command:** `python3 .specify/scripts/validate_feature_package.py`
- **Stop:** stop if a blocker in SPEC section 5 is unresolved or a contract change lacks owner promotion.

## 012-T02 - Implement take-control over a running turn

- **Status:** NOT STARTED
- **Depends on:** T01
- **Owner:** Agent Runtime and Conversation
- **Allowed paths:** conversation turn supervisor and storage settlement
- **Contract:** 001 cancel/quiescence authority plus this SPEC LLD
- **Change intent:** After T01 is accepted and the Feature 001 Stop/quiescence fixes (A-25) land, issue the cancel path, observe native quiescence outside any transaction, then atomically settle the turn `CANCELLED` and append an immutable `OWNER_TOOK_CONTROL` control event, preserving the stopped turn's committed content as owner-editable context.
- **Acceptance:** AC-001, AC-002, AC-004, AC-008, AC-010, AC-011, AC-013; take-control settles only after observed quiescence, commits the control event atomically with settlement, rejects a terminal turn, keeps a racing completion authoritative, redacts the native handle, reloads owner-in-control state after restart, and bounds the quiescence wait with a finite deadline.
- **Command:** `cargo test -p litecoworkd --locked conversation_take_control`
- **Stop:** stop if a blocker in SPEC section 5 is unresolved or a contract change lacks owner promotion.

## 012-T03 - Implement correlated side-chat

- **Status:** NOT STARTED
- **Depends on:** T01
- **Owner:** Conversation
- **Allowed paths:** conversation message commit and context package builder
- **Contract:** SPEC LLD and CONTEXT owner
- **Change intent:** Add the `OWNER_SIDE_NOTE` message kind correlated to a non-terminal turn id, commit it as ordinary bounded history, and exclude it from that turn's `ConversationContextPackage` so it never enters the native prompt.
- **Acceptance:** AC-003, AC-005, AC-006, AC-007, AC-009; a side-note commits correlated to a running turn and displays without mutating it, is rejected against a terminal turn and when over budget, is excluded from the native prompt, and never admits a second concurrent active turn.
- **Command:** `cargo test -p litecoworkd --locked conversation_side_chat`
- **Stop:** stop if a blocker in SPEC section 5 is unresolved or a contract change lacks owner promotion.

## 012-T04 - Render the truthful driver indicator

- **Status:** NOT STARTED
- **Depends on:** T02-T03
- **Owner:** Desktop UI
- **Allowed paths:** conversation transcript and composer
- **Contract:** contracts/ui-behavior.md
- **Change intent:** Derive and render the driver indicator (agent turn running / owner in control / side-chat visible) from committed turn status and the latest control event, with accessible state labels, and surface the take-control and side-chat affordances only while a turn is non-terminal.
- **Acceptance:** AC-012; the transcript shows a truthful driver indicator sourced from committed status plus the latest control event, exposes co-edit affordances only on non-terminal turns, and never infers the driver from the selected agent.
- **Command:** `python3 .specify/scripts/validate_feature_package.py`
- **Stop:** stop if a blocker in SPEC section 5 is unresolved or a contract change lacks owner promotion.
