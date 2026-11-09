# Plan - Professional Artifacts One Format

**Feature:** 007 | **Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.
**Spec revision:** 1.0 | **Release target:** V1 (Windows x64 first; Linux x64 second)

## Phases

1. Contract promotion (TASKS T01): promote B-21 engine pin, B-30 create, B-31 export receipt, the capability matrix and Conversation origin into ARTIFACTS-EVIDENCE, API, EVENTS, SCHEMAS with `docs/schemas/operator-api.openapi.yaml`, `domain-event.schema.json` (`artifact.exported.v1`), `error-codes.schema.json` and the next immutable `docs/schemas/sqlite-vNN.sql`.
2. Storage and format (T02-T06): atomic create, Markdown adapter, Markdown version publish, capability matrix producer, export receipt.
3. Operator routes and Tauri bridge (T07); Conversation association (T08); imported source class (T12).
4. UI (T09-T11): Library and Workbench create/edit/export, read-only/unsupported rendering for DOCX/XLSX/PPTX/PDF, stored-version compare.
5. Failure/recovery (T13), cross-layer trace (T14), qualification of E2E-07 on both rows (T15), documentation sync (T16), package VERIFIED (T17).

## Dependency DAG

- T01 blocks every implementation task.
- T02 and T03 run in parallel after T01; T04 needs both. T05 needs only T01.
- Task-generated source class waits for Feature 005 Task output; the user-authored Markdown path does not.
- External side effects (format engine, native picker, local file write) never run inside the transactions declared in WIRING.yaml.

## Owners and file scope

Owners: Artifacts, Presentation, Storage, Desktop UI. Allowed implementation paths are listed per task in TASKS.md. The active source tree is in apps/ and crates/. Changes stay inside task-owned paths and require a clean selected worktree and source/contract/UI/test reconciliation.

## Migration, rollout, rollback

T01 adds `artifact_export_receipts` and nullable `source_conversation_id` / `source_message_id` columns on artifacts (accepted target interface, not yet in source) in the next free immutable `docs/schemas/sqlite-vNN.sql` (latest today is `sqlite-v28.sql`; the number is assigned at promotion because other packages also add migrations). The migration is additive. Rollback is revert to the previous committed revision and refuse create/export admission that needs the new fields; committed receipts and versions are never rewritten.

Rollout: the capability matrix ships first so the UI can hide unsupported controls before new write paths exist.

## Risks

- Two Markdown dialects (daemon validator versus the hand-written UI preview) drifting apart; T03 and T09 test both against one fixture set.
- Presenting a PDF or Office preview as editing.
- Local-export receipt destination class not confirmed by Product (B-31 caveat) delaying T06.
- Collapsing distinct states into one UI boolean.

## Verification gates

- `python3 .specify/scripts/validate_feature_package.py`
- TASKS.md commands, including denial, replay, restart and crash cases
- CODE, SYSTEM and owner USER evidence for E2E-07 on each release-matrix row
