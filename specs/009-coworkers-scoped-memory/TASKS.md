# Tasks - Coworkers and Scoped Memory

**Feature:** 009 | **Status:** DRAFT - do not execute before SPEC is ACCEPTED/LOCKED.

**Spec revision:** 0.2

Current evidence is recorded in SPEC §4 and RESEARCH.md. The tasks remain a target implementation plan; the existing Coworker configuration routes do not satisfy scoped Conversation, Coworker-owned ContextDocument or learning acceptance.

## 009-T01 - Lock minimal memory and quick-create binding

- **Status:** BLOCKED
- **Depends on:** NONE
- **Owner:** Product and Coworkers
- **Allowed paths:** this package
- **Contract:** SPEC §5
- **Change intent:** Resolve B-24 and B-25.
- **Acceptance:** B-24 memory-record/retention/purge decision and B-25 Quick Create lead-binding decision are recorded in SPEC by the named owners; AC-001/AC-002 target semantics are explicit; AC-012 Quick Create partial-failure safety (Coworker created for safe retry) is specified.
- **Command:** `python3 .specify/scripts/validate_feature_package.py`
- **Stop:** stop if a blocker in SPEC section 5 is unresolved or a contract change lacks owner promotion.

## 009-T02 - Coworker identity and scoped documents

- **Status:** NOT STARTED
- **Depends on:** T01
- **Owner:** Coworkers and Storage
- **Allowed paths:** coworker service
- **Contract:** COWORKERS-TARGET and ADR-0024
- **Change intent:** Create, scope and purge without grants.
- **Acceptance:** AC-002 Coworker-owned first-chat path; AC-004 revoke/purge fence; AC-005 stale Coworker revision conflict; AC-008 owner-authored memory provenance (MemoryRecordMetadata.status, never a forged LearningCandidate); AC-009 memory grants no capability; AC-010 unpromoted owner kind is rejected; AC-011 purge retains a receipt without bytes; AC-013 over-bound memory is rejected with a typed error and never truncated; AC-014 memory text stays inert data that grants nothing and leaks no raw path into shared events; AC-015 concurrent Add Memory and revoke/purge serialize so the fence stays authoritative and purged bytes never resurface. Requires the Conversation and Resource owner contracts, not just Coworker CRUD.
- **Command:** `cargo test -p storage-sqlite --locked coworker_scope`
- **Stop:** stop if a blocker in SPEC section 5 is unresolved or a contract change lacks owner promotion.

## 009-T03 - Scoped retrieval on restart

- **Status:** NOT STARTED
- **Depends on:** T02
- **Owner:** Conversation and Memory
- **Allowed paths:** retrieval
- **Contract:** CONTEXT.md
- **Change intent:** Filter by scope across restart.
- **Acceptance:** AC-003 scoped retrieval excludes other Coworkers and ordinary Conversations; AC-006 restart restores the Coworker chat without cross-scope leakage; AC-016 scoped retrieval stays bounded to the Coworker scope within the PROPOSED budget and never full-corpus scans. Verify exact context pins/receipts and revoke races.
- **Command:** `cargo test -p litecoworkd --locked coworker_retrieval_fence`
- **Stop:** stop if a blocker in SPEC section 5 is unresolved or a contract change lacks owner promotion.

## 009-T04 - Optional Coworker UI

- **Status:** NOT STARTED
- **Depends on:** T03
- **Owner:** Desktop UI
- **Allowed paths:** coworkers and new chat
- **Contract:** ui-behavior
- **Change intent:** New Chat stays available without a Coworker.
- **Acceptance:** AC-001 ordinary New Chat remains available without a Coworker; AC-002 Quick Create opens the first Coworker-owned chat; AC-007 the Coworker surface keeps New Chat available and renders memory review/revoke as accessible states; existing Coworker selection in the Task composer does not satisfy either Conversation requirement.
- **Command:** `python3 .specify/scripts/validate_feature_package.py`
- **Stop:** stop if a blocker in SPEC section 5 is unresolved or a contract change lacks owner promotion.
