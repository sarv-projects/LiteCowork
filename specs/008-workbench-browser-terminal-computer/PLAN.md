# Plan - Workbench Browser Terminal and Computer

**Feature:** 008 | **Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.
**Spec revision:** 1.0

## Phases

1. SPEC section 5 blockers B-22, B-23 and B-32 are ACCEPTED (2026-11-09). Workbench truthfulness baseline (A-51) is verified in source; finish Activity fallthrough and the Computer UNAVAILABLE surface.
2. Promote accepted browser, terminal, Activity and Diff interfaces in the canonical owner docs and `docs/schemas/` (OpenAPI, events, error codes, next immutable `sqlite-vNN.sql`) in one reviewed change.
3. Browser lane (Windows x64 row first): storage, managed Chromium provider, service/routes/bridge, UI, recovery.
4. Terminal lane (Linux x64 row first, after Feature 005 admits a bubblewrap Environment): storage, discovery and PTY, service/routes/bridge, UI, recovery. Windows ConPTY waits on Feature 005 B-35.
5. Activity and Diff lanes (independent of providers): read projection and exact-revision diff.
6. Cross-layer traces, packaged E2E-08 qualification on each row, documentation sync, VERIFIED transition.

The ordered task list is [TASKS.md](TASKS.md).

## Dependency DAG

- T01 (contract promotion) blocks every implementation task except T02 (UI truthfulness), which may land first.
- Browser T03-T07, terminal T08-T13, Activity T14 and Diff T15 are independent lanes; T16-T19 close the package.
- External side effects never run inside the transactions declared in WIRING.yaml.
- Feature 001 remains a prerequisite wherever this slice needs a committed Conversation turn.

## Owners and file scope

Owners: Desktop shell, Environments, Browser provider, Trust, Desktop UI.
Allowed implementation paths are listed per task. The active source tree is in apps/ and crates/. Changes remain limited to the task-owned paths and require package acceptance, a clean selected worktree, and source/contract/UI/test reconciliation.

## Migration, rollout, rollback

Browser session/lease/receipt and terminal session/receipt records need one new immutable SQLite migration (the next `docs/schemas/sqlite-vNN.sql` after the highest present at implementation time; `sqlite-v28.sql` is highest on 2026-11-09), promoted by T01 and implemented by T03/T08. Activity and Diff add no tables. Rollback is revert to the previous committed revision and refuse new admission that needs the new field.

## Risks

- Implementing a provider before its contract is promoted (T01).
- Claiming terminal qualification on Windows before Feature 005 B-35, or from WSL2.
- Treating archive CODE as current qualification.
- Collapsing distinct states into one UI boolean.

## Verification gates

- `python3 .specify/scripts/validate_feature_package.py`
- Owner acceptance recorded 2026-11-09
- TASKS.md commands and denial, replay, restart and cancel cases
- Packaged E2E-08 on each release-matrix row
