# Plan - Safe Task Path

**Feature:** 005 | **Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.

## Phases

1. Contract promotion (T01): owner docs, OpenAPI, error codes, events and the next immutable SQLite migration in one change.
2. Foundations in parallel (T02-T07, T12): type/probe consolidation, plan acceptance storage, Linux bubblewrap provider and attestation, planning dispatch, B-18 replace primitive, Trust/grant wiring, Conversation promotion.
3. Execution path (T08-T11): Attempt admission, reconciliation and verification, Operator routes and Tauri bridge, cancel/crash/restart recovery.
4. UI (T13-T14): Work execution panel and the Windows-row UNAVAILABLE posture.
5. Linux-row qualification (T15) with the desktop USER path in [ACCEPTANCE-AND-E2E-PLAN.md](../../docs/ACCEPTANCE-AND-E2E-PLAN.md) E2E-04.
6. B-35 Windows provider (T16 owner decision, T17 implementation, T18 Windows-row qualification), after the Linux slice.
7. Documentation sync (T19) and VERIFIED (T20).

## Dependency DAG

- T01 blocks every implementation task; T16 is an owner decision that only blocks T17 and T18. The full DAG is in [TASKS.md](TASKS.md#dependency-order).
- External side effects never run inside the transactions declared in WIRING.yaml.
- Feature 001 remains a prerequisite wherever this slice needs a committed Conversation turn.

## Owners and file scope

Owners: Task Runtime, Environments, Trust, Storage, Desktop UI.
Allowed implementation paths are listed per task. The active source tree is in apps/ and crates/. Changes remain limited to the task-owned paths and require package acceptance, a clean selected worktree, and source/contract/UI/test reconciliation.

## Migration, rollout, rollback

No migration has shipped for this package yet. T01 promotes one forward-only immutable migration (`docs/schemas/sqlite-v29.sql` or the next unused number at implementation time; `sqlite-v28.sql` is defined but unwired per docs/STORAGE.md). Rollback is revert to the previous committed revision and refuse new admission that needs the new field.

## Risks

- Implementing B-35 work before the owner accepts B-35.
- Shipping a Windows build that admits an Attempt with a Job Object as the only boundary.
- Treating archive CODE as current qualification.
- Collapsing distinct states into one UI boolean.

## Verification gates

- `python3 .specify/scripts/validate_feature_package.py`
- Owner review recorded 2026-11-09 for B-17/B-18; B-35 pending
- TASKS.md commands and denial, replay, restart and cancel cases
