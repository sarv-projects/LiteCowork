# Plan - Native Agent Install, Auth and Binding

**Feature:** 002 | **Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.
**Spec revision:** 1.0

## Phases

SPEC section 5 blockers B-11 and B-12 and the A-35 screen split were owner-accepted on 2026-11-09. Phases now follow TASKS.md:

1. P1 prerequisite: make the OpenCode v2.0.32 profile truthfully bindable (002-T01), so Feature 001 Send is reachable from the UI.
2. Contract promotion: AGENT-CONTROL, OpenAPI, error codes, events and the next immutable `sqlite-vNN.sql` together (002-T02).
3. Module registry, descriptor storage and descriptor producer (002-T03 to 002-T05), then the fresh-descriptor enable gate (002-T06).
4. Native sign-in observation and credential slots (002-T07, 002-T08).
5. Composer options/commands and Settings operations wired into the Feature 011 screen (002-T09, 002-T10), honest install/update unavailability (002-T11).
6. Claude Code CLI as the second module (002-T12), failure/recovery (002-T13), cross-layer trace (002-T14).
7. Qualification on the release matrix, documentation sync and VERIFIED (002-T15 to 002-T17).

Run the commands named per task and the desktop USER path in [ACCEPTANCE-AND-E2E-PLAN.md](../../docs/ACCEPTANCE-AND-E2E-PLAN.md) (E2E-01 binding portion; E2E-02 with Feature 004).

## Dependency DAG

- The local DAG is in TASKS.md "Dependency order". T01 and T02 have no local predecessor; T01 is the first implementation change.
- External side effects never run inside the transactions declared in WIRING.yaml.
- Feature 001 remains a prerequisite wherever this slice needs a committed Conversation turn.

## Owners and file scope

Owners: Agent Runtime, Security/Trust, Desktop UI, Product.
Allowed implementation paths are listed per task. The active source tree is in apps/ and crates/. Changes remain limited to the task-owned paths and require package acceptance, a clean selected worktree, and source/contract/UI/test reconciliation.

## Migration, rollout, rollback

Acceptance authorizes the migrations named in TASKS.md (Runtime-local descriptor and auth-observation storage in 002-T04 and 002-T07). Each uses the next unused immutable `docs/schemas/sqlite-vNN.sql` (v28 is the highest present on 2026-11-09) and lands with its owner doc, schema and tests in one change. No migration exists yet. Rollback is revert to the previous committed revision and refuse new admission that needs the new field.

## Risks

- Flipping the OpenCode offer to compatible without also fixing the provider key, executable resolver and version pin (SPEC §4a), which would expose a binding that cannot dispatch.
- Treating archive CODE as current qualification.
- Collapsing distinct states into one UI boolean.

## Verification gates

- `python3 .specify/scripts/validate_feature_package.py`
- Owner acceptance recorded 2026-11-09; owner USER sign-off still required for VERIFIED
- TASKS.md commands and denial, replay, restart and cancel cases
