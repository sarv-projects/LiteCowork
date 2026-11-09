# Plan - Desktop Shell IPC Settings and Runtime Health

**Feature:** 011 | **Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.
**Spec revision:** 1.0 | **Delivery wave:** 0 (foundation), with the stop/update gate closing after Features 001 and 005 settlement.

## Phases

1. SPEC section 5 blockers B-28, B-29, B-33 and A-35 are ACCEPTED (2026-11-09).
2. Promote the accepted gap interfaces (renameWorkspace, stop preview payload, update quiescence gate, power assertion, local preference store) in the canonical owner docs and `docs/schemas/` in one change.
3. Wave 0 foundation: Windows named-pipe DACL qualification, Linux socket qualification, IPC bounds, Runtime health honesty, inert-control removal, local preferences, keyboard/focus baseline.
4. Settings owner work: Workspace rename/archive, Settings -> Agents screen over Feature 002 operations, Notifications link to Feature 010, keep-awake.
5. Stop/update gate after Feature 001 Stop/quiescence and Feature 005 Effect/lease settlement.
6. Failure/recovery, cross-layer traces, packaged qualification on Windows x64 then Linux x64, documentation sync, VERIFIED transition.

The ordered task list is [TASKS.md](TASKS.md).

## Dependency DAG

- T01 (contract promotion) blocks tasks that add or change an interface; T02-T07 and T11 only harden existing code and may start in parallel.
- T13 and T14 wait on Feature 001 and Feature 005 settlement.
- External side effects never run inside the transactions declared in WIRING.yaml.
- Feature 001 remains a prerequisite wherever this slice needs a committed Conversation turn.

## Owners and file scope

Owners: Runtime, Operator IPC, Desktop UI, Product.
Allowed implementation paths are listed per task. The active source tree is in apps/ and crates/. Changes remain limited to the task-owned paths and require package acceptance, a clean selected worktree, and source/contract/UI/test reconciliation.

## Migration, rollout, rollback

Workspace rename needs no migration: the Workspace record already has `name` and `version` (`docs/schemas/sqlite-v1.sql`, OpenAPI `Workspace`). Local preferences use a desktop-local file, not SQLite. A later schema change uses the next immutable SQLite migration. Rollback is revert to the previous committed revision and refuse new admission that needs the new field.

## Risks

- Mounting the stop route before settlement is proven.
- Counting WSL2 runs as Windows or Linux evidence.
- Treating archive CODE as current qualification.
- Collapsing distinct states into one UI boolean.

## Verification gates

- `python3 .specify/scripts/validate_feature_package.py`
- Owner acceptance recorded 2026-11-09
- TASKS.md commands and denial, replay, restart and cancel cases
- Desktop stop/update qualification gate journey on each row
