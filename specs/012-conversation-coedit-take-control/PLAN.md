# Plan - Conversation co-edit and take-control

**Feature:** 012 | **Status:** DRAFT

## Phases

1. Resolve SPEC section 5 blocker B-34 with the named owners and confirm the Feature 001 Stop/quiescence fixes (register A-25) are scheduled. No code.
2. Promote any new message kind, control-event type or projection field in the canonical owner (STATE-MACHINES/EXPERIENCE/EVENTS) and `docs/schemas/` together if the locked design needs it.
3. Implement tasks in dependency order on a recovered source tree after ACCEPTED/LOCKED, reusing the Feature 001 cancel/quiescence authority rather than duplicating it.
4. Run the cargo filters in TASKS.md and the desktop USER path in [ACCEPTANCE-AND-E2E-PLAN.md](../../docs/ACCEPTANCE-AND-E2E-PLAN.md).

## Dependency DAG

- T01 blocks later tasks.
- External side effects (native stop/quiescence) never run inside the transactions declared in WIRING.yaml.
- Feature 001 turn settlement and an effective Stop/quiescence path are prerequisites for take-control.

## Owners and file scope

Owners: Conversation, Agent Runtime, Product. Allowed implementation paths are listed per task. The active source tree is in apps/ and crates/. Changes remain limited to the task-owned paths and require package acceptance, a clean selected worktree, and source/contract/UI/test reconciliation.

## Migration, rollout, rollback

No migration ships in DRAFT. A later control-event/side-note store change uses the next immutable SQLite migration. Rollback is revert to the previous committed revision and refuse new co-edit admission that needs the new field.

## Risks

- Implementing from this DRAFT before B-34 closes or before the A-25 Stop/quiescence defects are fixed.
- Letting side-chat text leak into the running turn's native prompt.
- Modeling co-edit as a new durable work type instead of a Conversation UX.

## Verification gates

- `python3 .specify/scripts/validate_feature_package.py`
- Owner review of B-34 and the RV-14 no-vendor-claim constraint
- After lock: TASKS.md commands and the take-control race, side-note exclusion, restart and single-flight cases
