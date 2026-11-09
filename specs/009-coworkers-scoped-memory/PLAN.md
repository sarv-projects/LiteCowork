# Plan - Coworkers and Scoped Memory

**Feature:** 009 | **Spec:** SPEC.md revision 1.0
**Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.
**Updated:** 2026-11-09. The authoritative ordered work list is [TASKS.md](TASKS.md) (T01 to T18).

## Phases

1. B-24 and B-25 were accepted by the repository owner on 2026-11-09 (SPEC §5): the minimal memory record is an owner-authored Coworker-scoped ContextDocument carrying `MemoryRecordMetadata.status`, and Quick Create starts UNBOUND until Feature 002 qualifies a binding. The automatic LearningCandidate pipeline belongs to Feature 020.
2. Promote every accepted machine-schema field in its canonical owner and `docs/schemas/` in the same reviewed change (TASKS.md T01). Since register A-18 the OpenAPI `ContextDocumentCreateMetadata.kind` enum is narrowed to PERSONAL_PROFILE and WORKSPACE_NOTES, matching `domain-workspace`. Re-widening it to accept COWORKER_NOTES is its own task (T04) that changes the OpenAPI schema, the `valid_context_document_metadata` validator and docs/DATA-MODEL.md together, per the A-18/B-24 sequencing rider.
3. Implement Conversation Coworker origin and first-chat flow separately from Task Coworker origin; then implement Coworker owner-scoped ContextDocument and Coworker-filtered retrieval with read/revoke/purge fences. Do not reuse Workspace-only retrieval as if it were Coworker scoped.
4. Implement the accepted minimal owner-authored memory record first. Automatic learning is explicitly outside this slice; its target pipeline remains in `docs/LEARNING.md` and needs separate storage/provider/service/API/event/UI promotion.
5. Implement tasks in dependency order on a recovered source tree after ACCEPTED/LOCKED.
6. Run the cargo filters in TASKS.md, package/contract validators and the desktop USER path in [ACCEPTANCE-AND-E2E-PLAN.md](../../docs/ACCEPTANCE-AND-E2E-PLAN.md).

## Dependency DAG

- T01 blocks later tasks.
- External side effects never run inside the transactions declared in WIRING.yaml.
- Feature 001 remains a prerequisite wherever this slice needs a committed Conversation turn.

## Owners and file scope

Owners: Coworkers, Conversation, Memory, Trust, Desktop UI.
Allowed implementation paths are listed per task. The active source tree is in apps/ and crates/. Changes remain limited to the task-owned paths and require package acceptance, a clean selected worktree, and source/contract/UI/test reconciliation.

## Migration, rollout, rollback

No migration has shipped for this package yet; the chain ends at `docs/schemas/sqlite-v28.sql`. Each accepted schema change (Conversation Coworker origin, capability assignment, responsibility proposals) lands as the next immutable `sqlite-vNN.sql` in the task that needs it. Rollback is revert to the previous committed revision and refuse new admission that needs the new field.

## Risks

- Implementing ahead of the TASKS.md dependency order or before the T01 contract promotion lands.
- Treating archive CODE as current qualification.
- Collapsing distinct states into one UI boolean.

## Verification gates

- `python3 .specify/scripts/validate_feature_package.py`
- Owner review
- After lock: TASKS.md commands and denial, replay, restart and cancel cases
