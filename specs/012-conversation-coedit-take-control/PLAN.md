# Plan - Conversation co-edit and take-control

**Feature:** 012 | **Spec revision:** 1.0
**Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.

## Phases

1. B-34 accepted by the owner on 2026-11-09 (done). Confirm the Feature 001 Stop/quiescence fix (register A-25, Feature 001 task T07) has landed before any implementation task starts.
2. Promote any new message kind, control-event type or projection field in the canonical owner (STATE-MACHINES/EXPERIENCE/EVENTS) and `docs/schemas/` together if the locked design needs it.
3. Implement tasks in dependency order (see [TASKS.md](TASKS.md)), reusing the Feature 001 cancel/quiescence authority rather than duplicating it.
4. Run the cargo filters in TASKS.md and the desktop USER path in [ACCEPTANCE-AND-E2E-PLAN.md](../../docs/ACCEPTANCE-AND-E2E-PLAN.md).

## Dependency DAG

- T01 blocks later tasks.
- External side effects (native stop/quiescence) never run inside the transactions declared in WIRING.yaml.
- Feature 001 turn settlement and an effective Stop/quiescence path are prerequisites for take-control.

## Owners and file scope

Owners: Conversation, Agent Runtime, Product. Allowed implementation paths are listed per task. The active source tree is in apps/ and crates/. Changes remain limited to the task-owned paths and require package acceptance, a clean selected worktree, and source/contract/UI/test reconciliation.

## Migration, rollout, rollback

Migration v32 (`docs/schemas/sqlite-v32.sql`, TASKS T05 storage layer, shipped 2026-11-10 and wired, `SCHEMA_VERSION` 32) holds the control-event and side-note records; v30 and v31 were taken by 001-T25 and 014-T03 first. Rollback is revert to the previous committed revision and refuse new co-edit admission that needs the new field.

## Risks

- Implementing before the A-25 Stop/quiescence defects are fixed in Feature 001.
- Letting side-chat text leak into the running turn's native prompt.
- Modeling co-edit as a new durable work type instead of a Conversation UX.

## Verification gates

- `python3 .specify/scripts/validate_feature_package.py`
- B-34 and the RV-14 no-vendor-claim constraint: owner-accepted 2026-11-09
- TASKS.md commands and the take-control race, side-note exclusion, restart and single-flight cases
