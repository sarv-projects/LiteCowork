# Spec, Build TASKS, and USER Test Traceability

**Updated:** 2026-11-06

**Purpose:** one crosswalk between **global build tasks** ([LITECOWORK-BUILD-TASKS.md](LITECOWORK-BUILD-TASKS.md)), **feature SDD tasks** (`specs/NNN-*/TASKS.md`), **acceptance IDs** (SPEC EARS), and **competitor USER cases** ([COMPETITOR-PROMOTED-USE-CASES.md](COMPETITOR-PROMOTED-USE-CASES.md)).

## Feature 001 — Conversation Session Supervisor

| Build task | Spec task | Primary AC-* | USER UC-* |
|---|---|---|---|
| BT-0.6, BT-1.* | 001-T01 | AC-001–004 blockers | UC-CHAT-01–05 |
| BT-1.2 | 001-T02, T06 | AC-009 | UC-CHAT-01 |
| BT-1.1 | 001-T03, T04 | AC-005–008 | UC-CHAT-01 |
| BT-1.1 | 001-T05 | AC-013–016 | UC-CHAT-06 |
| BT-1.3 | 001-T07 | AC-010–011, 017–019 | UC-CHAT-01, UIV1-016–022 |
| BT-1.4 | 001-T08 | AC-012 | FL01–FL05 |
| BT-1.5, BT-12.* | 001-T09 | AC-001–019 all | UC-CHAT-01 + full 001 EARS |

> The `UIV1-016–022` citation above is a provenance pointer to the archived legacy catalog only; the legacy `UIV1-xxx` namespace is retired as an active acceptance authority (register [A-39](CONTRADICTION-AND-ALIGNMENT-REGISTER.md)). The live acceptance obligation is the referenced flow/AC IDs, per [V1-UI-ACTION-STATE-LEDGER.md](V1-UI-ACTION-STATE-LEDGER.md).

**Spec package files (all required for lock):**

| File | Lines (approx) | Role |
|---|---:|---|
| [specs/001-.../README.md](../specs/001-conversation-session-supervisor/README.md) | 28 | Scope, owners, links |
| [SPEC.md](../specs/001-conversation-session-supervisor/SPEC.md) | 354 | EARS, HLD/LLD, blockers B-01–B-10 |
| [PLAN.md](../specs/001-conversation-session-supervisor/PLAN.md) | 106 | Phases, DAG |
| [TASKS.md](../specs/001-conversation-session-supervisor/TASKS.md) | 161 | 001-T01–T09 executable commands |
| [DATA-MODEL.md](../specs/001-conversation-session-supervisor/DATA-MODEL.md) | 78 | Records, B-04 boundary |
| [RESEARCH.md](../specs/001-conversation-session-supervisor/RESEARCH.md) | 101 | Evidence, rivals |
| [WIRING.yaml](../specs/001-conversation-session-supervisor/WIRING.yaml) | 236 | Components, transactions |
| [contracts/api-events.md](../specs/001-conversation-session-supervisor/contracts/api-events.md) | 63 | OpenAPI/event binding |
| [contracts/errors-recovery.md](../specs/001-conversation-session-supervisor/contracts/errors-recovery.md) | 78 | Typed failures |
| [contracts/ui-behavior.md](../specs/001-conversation-session-supervisor/contracts/ui-behavior.md) | 93 | UI flows/states/commands |

**Validator:** `python3 .specify/scripts/validate_feature_package.py` (the validator walks every package under specs/; it takes no package argument)

## Features 002–012

Each package is a DRAFT (README, SPEC, PLAN, TASKS, DATA-MODEL, RESEARCH, WIRING, and required contracts). Status is not ACCEPTED/LOCKED. Blockers live in the owning SPEC. Task ids are package-specific; always read the current TASKS.md rather than assume a fixed upper bound. Feature 002 now includes 002-T06 for a descriptor-bound model/effort picker (blocked on exact native dispatch). Feature 010 includes 010-T05 for future qualified external WorkflowBackend Routine mapping (blocked) and 010-T06 for provider-authoring evaluation (deferred; not V1 authorization).

| SDD | Build phase | USER themes | Blockers to close before lock |
|---|---|---|---|
| 002 | BT agent matrix | UC agent install/auth | B-11, B-12, shared first route (B-11/B-12 have PROPOSED resolutions in specs/002 §5, still open pending owner) |
| 003 | P2 / BT roots | UC files and citations | B-13, B-14 (PROPOSED resolutions in specs/003 §5, still open pending owner) |
| 004 | P3 | UC relay A→B→A | B-15, B-16 (PROPOSED resolutions in specs/004 §5, still open pending owner) |
| 005 | P4 | UC one safe task edit | B-17, B-18 (PROPOSED resolutions in specs/005 §5, still open pending owner) |
| 006 | P5 | UC one connection write | B-19, B-20 (PROPOSED resolutions in specs/006 §5, still open pending owner) |
| 007 | P6 | UC one professional format, Artifact create and export | B-21, B-30, B-31 (PROPOSED resolutions in specs/007 §5, still open pending owner) |
| 008 | P7 | UC browser, terminal and optional Computer use | B-22, B-23, B-32 (PROPOSED resolutions in specs/008 §5, still open pending owner; managed-Chromium adapter disables ambient-CDP/profile-copy) |
| 009 | P8 | UC optional coworker chat and scoped memory | B-24, B-25 (PROPOSED resolutions in specs/009 §5, still open pending owner, aligned to the single A-34 learning vocabulary); T02 Coworker origin + owner-scoped ContextDocument; T03 retrieval/restart fence; T04 optional New Chat/first Coworker chat |
| 010 | P9 | UC schedule exactly once; future provider-owned workflow mapping | B-26, B-27 (PROPOSED resolutions in specs/010 §5, still open pending owner; 010 owns the A-35 quiet-hours surface); T05 depends on qualified Features 005/006; T06 deferred |
| 011 | P0–P11 shell | UC settings honesty and health | B-28, B-29, B-33 (PROPOSED resolutions in specs/011 §5, still open pending owner; 011 owns the A-35 Settings->Agents screen and links 010 for quiet-hours) |
| 012 | P1 chat | UC co-edit / take-control on one Conversation | B-34 (PROPOSED resolution in specs/012 §5, still open pending owner; maps co-edit onto a Conversation take-control/side-chat UX, no new durable-Task type); research gate RV-14 (Claude Duet, URL unresolved, cited as RESEARCH); 012-T02 take-control depends on the Feature 001 Stop/quiescence fixes (register A-25) |

When an owner resolves a blocker, edit that SPEC and leave status DRAFT until explicit acceptance.

## .specify templates (copy source for new packages)

| Template | Use |
|---|---|
| [.specify/templates/SPEC.md](../.specify/templates/SPEC.md) | Full SPEC skeleton |
| [.specify/templates/TASKS.md](../.specify/templates/TASKS.md) | Task ID discipline |
| [.specify/templates/contracts/ui-behavior.md](../.specify/templates/contracts/ui-behavior.md) | UI behavior contract |
