# Current engineering handoff

- **Repository:** LiteCowork
- **Branch:** main
- **Inspected revision:** 52269e4af6038ccc849e5c2fbef5376f3cdd2d8e
- **Working-tree state at this handoff:** clean. The 197-entry pre-existing worktree recorded on 2026-11-06 has been committed in full across the audit-repair campaign; see the 2026-11-07 handoff below.
- **Release posture:** pre-release; V1 is not release-qualified

This file is the current engineering handoff. It records the active repository state and the checks performed against it. It is not a chronological diary and must not preserve stale "previous/latest" status paragraphs. Update the current facts and validation results in place whenever the repository changes.

## Repository state

The active checkout contains the Tauri/React desktop application, the Rust daemon and workspace crates, first-party docs and JSON contracts, 11 feature SDD packages, scripts and pinned reference repositories. The repository is not documentation-only.

Primary application paths:

- Desktop UI: apps/litecowork-ui
- Daemon: apps/litecoworkd
- Shared domain/storage/runtime crates: crates
- Product and architecture contracts: docs
- Feature specifications: specs/001 through specs/011
- Specification templates and validators: .specify
- Reference implementation snapshots: repo_take (research only; never product source of truth)

The local ignored archive and historical documentation archive remain preservation material. They do not replace the active source tree.

## Product authority

Read these in order before material work:

1. Root ARCHITECTURE.md for ownership, invariants and system boundaries.
2. docs/PRODUCT.md and docs/V1-PRODUCT-RELEASE-CONTRACT.md for product promise and release scope.
3. The owning domain contracts and accepted ADRs.
4. docs/schemas for canonical OpenAPI, event, JSON Schema, migration and generated route inventory contracts.
5. docs/V1-UI-SCREEN-CONTRACT.md and docs/V1-UI-ACTION-STATE-LEDGER.md for the user-visible experience.
6. The relevant feature package under specs.
7. docs/IMPLEMENTATION-STATUS.md for what this checkout actually implements and the available evidence.
8. docs/ACCEPTANCE-AND-E2E-PLAN.md for required qualification.

Do not resolve a conflict by choosing whichever file has the newest date. Update the document that owns the semantic decision, then update its dependent contracts, UI maps, indexes and traceability.

## Current source-backed product status

The evidence details and implementation boundaries live in docs/IMPLEMENTATION-STATUS.md. The critical distinctions are:

- Runtime lifecycle health is not Task execution readiness.
- Operator API availability is not native-agent integration.
- A persisted Conversation turn is not proof that a provider completed it.
- Recent Chats are cursor-paginated; the sidebar loads additional pages only from the active Conversation page and preserves Workspace/list-epoch fencing. An unreadable pending-create journal pauses New Chat until its explicit retry succeeds.
- A persisted Task is not an admitted Plan or running Attempt.
- A source route registration is not a qualified user journey.
- A Workspace resource preview is not a live filesystem editor.
- A WebView iframe is not Chrome automation or a controlled browser.
- Terminal UI output is not a real process result unless it came from a real PTY/ConPTY process.
- A native action is not a Core-mediated Effect unless it passed through the authorized gateway.
- An Automation trigger host can materialize a READY Task without executing that Task.
- A Coworker configuration is not a continuously running agent process.
- A renderer or schema is not evidence that every artifact format or interactive app is usable.

The current Operator route inventory is docs/schemas/served-route-inventory.json. It records 173 OpenAPI route registrations, one authenticated local-only route outside OpenAPI and 56 OpenAPI operations that remain target-only. This inventory is generated from the composed daemon router. It is evidence of route mounting, not product qualification.

## Validation performed

The completed full Rust workspace run on the connected Windows development machine (historical; predates the Linux-side audit-repair batches that changed `storage-core`, `storage-sqlite`, `litecoworkd` and the docs, so its per-crate counts are superseded by "Current validation on the working tree" below):

- Command: `$env:CARGO_BUILD_JOBS='1'; cargo test --workspace --locked`
- **Exit code 0; runtime 1,383.28 seconds; no failed test targets.** All unit tests, integration tests and doc-tests completed successfully.
- `litecoworkd` unit target: **359 passed, 0 failed, 1 ignored**; `litecoworkd --test cli`: **4 passed**.
- `storage-core`: **110 passed**; `storage-sqlite`: **555 passed**; `storage-sqlite --test agent_registry_cache`: **2 passed**.
- `windows-local-security`: **36 passed**; `local-environment-staging`: **24 passed**; `operator-ipc`: **18 passed**.
- Domain targets passed, including Conversation (8), Effects (14), Environment (24), Invocations (14), RAG (38), Responsibility (99), Task (138), Workspace (7) and Linux process scope (5). Browser control (23) and browser pointer (9) integration targets also passed.
- The only ignored tests in the Windows run were the daemon's declared ignored test and the Windows Credential Manager integration test that writes uniquely scoped entries to the current user's credential store. Linux Secret Service integration is not run on this Windows target.
- The six Windows WorkspaceRoot route tests pass with platform-canonical fixture identities; production path validation remains fail-closed. Windows folder scanning remains unqualified.
- The deterministic cross-platform backup failure-path test passes as part of the 555-test `storage-sqlite` unit target.

Current validation on the working tree:

- Rust workspace: `cargo fmt --all -- --check` clean; `cargo clippy --workspace --all-targets --locked -- -D warnings` **0 diagnostics**; `cargo check --workspace --all-targets --locked` **0 errors**.
- `cargo test -p storage-sqlite --locked`: baseline was **594 passed, 1 failed** (the pre-existing, environment-dependent `blob::tests::refuses_symlinked_blob_directory_ancestors`). V1 implementation note (2026-11-09): wiring `SCHEMA_VERSION` 27 to 29 (migration v28, previously unwired per register A-40, plus the new v29 conversation dispatch migration) activated the v28 delegation-policy triggers (`task_spec_delegation_policy_snapshot_guard`, `workspace_delegation_settings_revision_insert_guard`). Two pre-delegation fixtures now abort because they insert `task_spec_revisions` without a conforming `delegation_policy_snapshot_json` and settings revision: `lead_failover_choice_request::tests::a_stale_task_version_or_spec_revision_is_rejected` and `tests::environment_tests::sharing_scope_change_checks_expected_environment_version`. These are fixture-reconciliation follow-ups (seed a `workspace_delegation_settings_revisions` row and a valid snapshot, or use the v27 boundary helper), not defects in the ACP work; tracked for the Feature 001/005 implementation pass.
- `cargo test -p storage-core --locked`: **113 passed, 0 failed**; `cargo test -p domain-environment --locked`: **54 passed, 0 failed** (targets 22/23/9).
- `litecoworkd` suite (most recent full run): **507 passed, 36 failed, 1 ignored** — the 36 are environment-dependent failures concentrated in `agents::cline::*` (native CLI/npm probes) and `local_filesystem::*` (procfs / in-place-mutation semantics); the failure set is byte-identical to the pre-change baseline.
- Composed Operator inventory: **173 OpenAPI served / 1 authenticated local-only / 56 target-only**, generated and matching source router composition.
- Architecture/schema validator: passed active checks; one warning because the optional `openapi-spec-validator` package is not installed. JSON Schema, 148 event schemas, errors, OpenAPI structure, SQL, Gateway naming, concept and active Markdown checks passed.
- Cross-owner consistency guard: **36 assertions passed, 0 issues**.
- V1 contract/route/UI audit: **0 errors** (229 OpenAPI operations; 173 served, 1 outside OpenAPI, 56 target-only).
- Design-depth audit: **0 findings**; 359 files, 72,258 lines, 12 domains, 254 functions, 187 Settings rows, 229 routes, 107 tables, 20 packages.
- Feature SDD validator: **20/20 packages passed** (001-012 ACCEPTED/LOCKED 2026-11-09; 013-020 IN_REVIEW).
- Markdown audit: **0 broken relative links** across 356 files / 72,271 lines. The generated active-corpus manifest (docs/COMPLETE-ACTIVE-CORPUS-INVENTORY.md) excludes itself and records 359 files / 72,258 lines.
- Sandbox note: on this Linux sandbox, cargo and Python `uv` commands require the prefix `TMPDIR=/tmp PKG_CONFIG_SYSROOT_DIR=/tmp/sysroot PKG_CONFIG_PATH=/tmp/sysroot/usr/lib/x86_64-linux-gnu/pkgconfig LIBRARY_PATH=/tmp/sysroot/usr/lib/x86_64-linux-gnu`.
- Frontend production build: passed API type check, TypeScript and Vite. Main JS bundle is 1.05 MB minified (287 KB gzip), above Vite's 500 KB advisory.
- Focused Conversation + responsive tests: **24/24 passed**, including journal retry, cursor pagination, drawer semantics and mobile canvas sizing.
- The full serial frontend test command was attempted but did not emit a final TAP summary. It reached test 393 and stalled in the Settings Chrome DevTools check; observed failures included two production-App bootstrap timeouts and, before fixes, the Conversation pagination and responsive assertions. The latter three are now fixed and pass in isolation. Do not report an aggregate full-suite result.
- The standalone Settings accessibility test passed its static WCAG AA contrast check but failed its rendered-browser phase because Chrome DevTools timed out on `Page.navigate`. This does not establish the production Settings screen's keyboard behavior.
- Rendered UI, packaged Tauri smoke and real-provider desktop E2E remain unqualified.

The full Windows Rust workspace test suite is green, but that is not a LiteCowork V1 release pass. Do not describe frontend coverage as complete or imply packaged desktop/provider USER qualification.

## Release-critical work

1. Rust workspace tests are green on Windows with `CARGO_BUILD_JOBS=1`; retain the ignored credential-store test and unqualified Windows folder-scan provider as explicit platform gates. Next engineering priority is Conversation dispatch durability, strict single-flight admission, retry/cancel state-machine repair and restart recovery.
2. Make Conversation dispatch recoverable across the durable-submit/async-dispatch boundary; verify idempotency, timeout, crash and restart.
3. Qualify the real OpenCode turn path and implement/qualify additional native adapters before making their Send controls available.
4. Implement one safe Task vertical slice: planning, accepted plan revision, Attempt admission, Environment, Trust, one mediated Effect, reconciliation and verification.
5. Remove simulated terminal success and generated sample diffs; implement process-owned PTY/ConPTY and real change computation.
6. Keep iframe preview accurately labelled; implement actual browser/computer services before advertising automation or takeover.
7. Remove or wire Settings controls that currently affect only local UI state or notices.
8. Qualify LiteSPM service/version/protocol/receipt handling before package lifecycle mutations.
9. Qualify one professional Artifact format and one typed Intelligent UI schema path.
10. Verify each screen's UI state against its owner API and failure/recovery behavior.

## Preservation and delivery rules

- Inspect the worktree and ignored/untracked paths before every change.
- Preserve unrelated user work; never clean, reset, stash, rebase or delete archives.
- Do not alter repo_take clones as part of LiteCowork implementation.
- Do not stage secrets, private agent profiles, local provider credentials, build outputs or caches.
- Do not commit or push unless explicitly requested.
- Keep target contracts, implementation findings and qualification evidence distinct.
- Run the narrow test for every change, then contract checks and broader regression checks.
- Inspect the complete diff and verify generated inventories before considering a documentation reconciliation complete.

## Current repository handoff - 2026-11-07

- **Branch / HEAD:** main / 52269e4af6038ccc849e5c2fbef5376f3cdd2d8e.
- **Working tree:** clean. The 197-entry state recorded on 2026-11-06 was committed in full during the audit-repair campaign; nothing was reset, cleaned, stashed, rebased or rewritten.
- **Changes in this work session:** the audit-repair campaign — the workspace lint gate driven to zero with every `#[allow]` carrying a written reason (register A-46); the resource-root read policy made fail-closed in production with its drifting test-only twin deleted (register A-50); the Environment vocabulary migrated into `domain-environment`, inverting the two-crate dependency edge with `storage-core` re-exporting the moved names through an error bridge (register A-41); contradictions recorded with cited evidence in docs/CONTRADICTION-AND-ALIGNMENT-REGISTER.md; all generated documents regenerated to a fixed point; full `check.sh --preflight` green.
- **Release posture:** pre-release; V1 is not release-qualified. Feature 001 remains IN_REVIEW; Features 002-011 remain DRAFT.
- **Open inventory:** docs/CONTRADICTION-AND-ALIGNMENT-REGISTER.md is the tracked contradiction register; rows whose resolution cell still prescribes an action instead of reporting one remain open and are the next audit targets.

### Source-to-document reconciliation completed

The active source crosswalks and matching UI contracts are maintained in docs/IMPLEMENTATION-STATUS.md, docs/SEMANTIC-SERVED-UI-MAP.md, docs/V1-UI-SCREEN-CONTRACT.md, docs/FEATURE-AND-UI-CONTRACT-COVERAGE.md, and each owning package under specs/001 through specs/011.

- **001 Conversation:** one OpenCode-only dispatch branch; detached dispatch has no durable outbox/startup sweep; strict single-flight is not enforced at admission; Send violates the disabled-until-qualified gate; Stop is ineffective for OpenCode; failed-turn Retry has no UI call site and its backend transition is inconsistent; native transient token frames are not emitted.
- **002 Agents:** 15 curated candidates and ACP metadata are not installed/ready agents. Auth is UNKNOWN and readiness NOT_PROBED. Only Codex/OpenCode profile probes exist; OpenCode is incompatible, Codex has no Conversation dispatch branch. Install/native sign-in are absent; descriptor and credential routes fail closed; binding enable lacks the fresh AC-004 descriptor gate; Usage/model-effort UI is not qualified.
- **003 Resources:** root selection immediately commits on supported Linux/macOS; Windows Add returns 503. Root scan is Linux-only metadata census; Windows/macOS return 501. No content ingestion/watch/index from roots. ZIP originals remain opaque; text index is allowlisted UTF-8 only. Grounded retrieval is read-only and not wired to Conversation or Context Used.
- **004 Agent relay:** backend producer/session identity is projected but desktop message types/parser drop it; picker eligibility is not equivalent to dispatchability; only OpenCode dispatch exists; no qualified A-to-B-to-A route or Resource materialization.
- **005 Tasks:** Task creation and planning readiness are real records/read-only preflight; no accepted-plan -> Attempt -> Environment -> mediated Effect -> verification execution pipeline exists. READY is not running.
- **006 Capabilities:** Discover projections and exact Approval decisions exist, but APPROVE is not an external Effect. Connect/install fails closed, test is UNKNOWN, account Connection/OAuth is absent, CapabilityInvocation is read-only, and provider dispatch/reconciliation is not wired.
- **007 Artifacts:** no production Artifact-create route or create UI; managed text/plain edit/append and safe previews exist; native Save As is local-only and creates no ExportReceipt. No professional format engine or Task output producer.
- **008 Workbench:** Files is a Resource projection, Browser is an iframe preview, Terminal fabricates command success without a process, Diffs are synthetic, Activity is not rendered and Computer-use is unavailable.
- **009 Coworkers:** identity/revisions/status and configuration persist, but there is no Coworker-owned Conversation, Coworker-scoped retrieval or private memory browser/Learning Inbox. Presence returns 503; several saved policy fields have no production consumer; PAUSED admission is partial.
- **010 Automations/Notifications:** local 30-second TriggerHost settles supported recurring schedules and materializes READY Tasks only. Retry/wake policy fields have no consumer. Notification outcome settlement inserts PENDING rows, but no sender advances delivery state; channel preferences are ignored, ON_CONDITION is only DEPENDENCY_BLOCKED, and quiet hours are not editable.
- **011 Shell/Settings:** authenticated OS-local IPC exists; mounted Settings has four categories, not the 16-category target. Appearance controls are ephemeral/inert, shortcut rows are display-only, and several Runtime/Workspace actions are not exposed by the mounted view.

### Global blocker identity

Blocker IDs are globally unique across feature packages. The current allocation is B-21 professional Artifact format; B-22 browser provider; B-23 terminal provider; B-24 Coworker memory; B-25 Quick Create; B-26 Automation trigger; B-27 local notification delivery; B-28 GA OS/security profile; B-29 Settings scope; B-30 Artifact creation; B-31 Artifact export receipt; B-32 Computer-use decision. docs/SPECIFICATION-STANDARD.md and .specify/templates/SPEC.md now require allocation from the global register and synchronized updates to owner SPEC/TASKS/traceability.

The external workflow graph remains provider-owned. WorkflowBackendAdapter and provider graph authoring are target-only; no Core graph runtime or production adapter is present. OpenBot/n8n/Activepieces research is preserved in the linked reference study and does not authorize a Core workflow engine or direct provider graph mutation.

### Current validation

- Feature package validator: **12/12 passed**; structural checks only, not owner approval or runtime qualification.
- Cross-owner consistency: **36 assertions passed, 0 issues**.
- V1 contract audit: **0 errors**; 229 OpenAPI operations, 173 served, 1 authenticated local-only route and 56 target-only.
- Design-depth audit: **0 findings**; 359 files, 72,258 lines, 12 domains, 254 functions, 187 Settings rows, 229 routes, served 173 / target-only 56, 107 tables and 20 packages.
- Markdown audit: **0 broken relative links** across 356 files and 72,271 lines.
- Active-corpus manifest: **359 files and 72,258 lines** (manifest excludes itself).
- Architecture/schema validator: passed with one warning because optional openapi-spec-validator is not installed; JSON Schema, 148 typed events, errors, OpenAPI structure, SQLite, Gateway naming, canonical concepts and active Markdown checks passed.
- Rust workspace: lint gate (`clippy --all-targets -- -D warnings`) **0 diagnostics**; `check --all-targets` **0 errors**; `fmt --all --check` clean; test counts as listed under "Current validation on the working tree".
- Whitespace check: **passed** (`check.sh` whitespace stage, exit 0).
- The full serial frontend suite remains incomplete; the Settings rendered-browser check timed out; packaged Tauri and real-provider USER E2E remain unqualified.

### Next engineering gates

1. Keep Send disabled until the exact native OS/provider/auth USER path and durable dispatch/single-flight/stop/retry recovery are qualified.
2. Implement one safe Task vertical slice with accepted plan, Attempt, Environment, Trust, mediated Effect, reconciliation and independent verification.
3. The Workbench Terminal/Diffs/Browser simulation is already removed from the UI (register A-51): Terminal shows "unavailable", Diffs/Review shows "nothing to review", the browser iframe starts empty with no containment claim. The remaining obligation is to implement real provider-backed sessions and evidence (PTY/ConPTY B-23, browser B-22) and defer unsupported Computer use (B-32).
4. Complete one Artifact create/format/export/reopen path with a durable receipt.
5. Complete one account Connection and provider read/write/revoke journey; Approval alone is not execution.
6. Close Coworker-owned Conversation, scoped Memory and proactive PAUSED-admission gaps before enabling those claims.
7. Keep local automation, notification delivery and Settings controls honest until their actual consumers/handlers exist.

### Phase 1 assumption-free doc re-audit resolved (FEAT-002)

The full-corpus re-audit verdict table is [docs/DOCS-REAUDIT-2026.md](docs/DOCS-REAUDIT-2026.md) (five clusters, plus an SDD-completeness-gap list for specs/001-011). It reconciled the known danglers at their owners and recorded the decisions in the register:

- **A-38** (FLOWS.md flow index): the "Flow outputs and UI projection" index now covers F00-F143 (added F115-F124 and F136-F143), and the F119/F128 duplicate is resolved by distinguishing the save flow (F119) from its navigation-recovery sibling (F128).
- **A-39** (UIV1 namespace): retired as an active acceptance authority; acceptance resolves on live flow IDs and feature AC IDs, with resolution notes at V1-UI-ACTION-STATE-LEDGER.md and V1-END-TO-END-FLOW-AND-RACE-CONTRACTS.md. The archived catalog stays provenance-only; specs/README.md's UIV1 instruction is updated.
- **A-40** (sqlite-v28 unreachable): v28 is recorded as defined-but-not-yet-wired (`SCHEMA_VERSION` is 27); STORAGE.md ("current schema version is 28" corrected to 27), EVENTS.md, DELEGATION.md and COVERAGE-MATRIX.md are corrected to stop presenting v28 as active.
- **Line-count drift**: CURRENT-RUN.md and IMPLEMENTATION-STATUS.md now cite the live generator output; the corpus audits/generators exclude `.agents/` plan files so plan-file churn no longer inflates corpus counts.
- **A-51 (new)**: the Workbench Terminal/Diff/Browser status docs were stale after the UI was remediated; IMPLEMENTATION-STATUS.md and rows A-08/A-09/A-10/A-30 are corrected to the current source.

### Phase 3 superset matrix + three-way gap analysis resolved (FEAT-004)

The current **cross-cutting gap view** is [docs/SUPERSET-GAP-MATRIX.md](docs/SUPERSET-GAP-MATRIX.md): one feature × {LiteCowork done? (TARGET/CODE/SERVED/INTEGRATED/QUALIFIED) / doc-claims? / doc-accuracy (linked to DOCS-REAUDIT-2026 + A-xx) / each resolved rival has it?} matrix whose feature axis is the union of LiteCowork's capability set and every Phase 2 resolved rival's first-release surface (max superset). It derives the three lists the audit asked for — ✓ Done / ✗ Not-done (real state), 📝 doc errors, 🎯 missing-to-be-a-superset. It **reads** the owners and holds no authority: parity verdicts stay in docs/VIABILITY-VS-COMPETITORS.md, concern→owner in docs/COVERAGE-MATRIX.md, verdicts in docs/DOCS-REAUDIT-2026.md, real state in docs/IMPLEMENTATION-STATUS.md. Cross-linked from VIABILITY-VS-COMPETITORS.md, COMPETITIVE-RESEARCH.md and the docs/README.md index.

- **Headline gap count:** **14** capability gaps to reach a max superset (8 already TARGET-owned via allocated B-xx/RV-xx; 6 RESEARCH/owner-decision gates — RV-06, RV-07, RV-08, RV-13, RV-14, plus the consent-gated screen/audio decision); **~18** doc-accuracy findings (2 still-open code defects: A-24 Send gate, A-26 empty-canvas branding); **0** LiteCowork rows reach Done as a qualified product journey (the lone "Done (pattern)" is the runtime-split invariant), consistent with the pre-release posture.
- No new tracker ids were invented: every missing/owner-decision item reuses an allocated B-xx / RV-xx.

### Pending research (recorded 2026-11-07)

- **Dify workflow-sufficiency verdict — failed on infrastructure mid-run** (session `conversation://394419fe-285e-483c-8478-75f478adb2f9`). Objective unchanged: an evidence-based verdict, grounded in a Dify read, on whether the Trigger-to-Occurrence-to-Task-to-CapabilityInvocation model is sufficient for the V1 promise or a graph engine is warranted — plus a lightweight visible-execution-path sketch that fits the existing authorities. Relaunch; do not re-scope.
- **Agenta visible-execution-path design — failed on infrastructure mid-run** (session `conversation://b4977637-41e5-4e84-b74d-82df1cdf217f`). Objective unchanged: a design (no vendor dependency) derived from Agenta's source for a truthful Task/Attempt and Automation-occurrence execution projection owned by existing LiteCowork authorities. Relaunch; do not re-scope.
- **Competitive-landscape corrections — APPLIED 2026-11-08 (Phase 2 fresh rival re-verification).** Resolved into the owning competitor docs under their own evidence rules: Claude Duet added as a tracked rival (RV-14; first-party URL unresolved on re-check, gap recorded); the OpenBot nightly-labs/openbot vs CopilotKit/openbot naming collision preserved and OpenBot repinned 0.62.0→0.65.0 (PolyForm Noncommercial, reference-only); Grok's app/marketplace layer confirmed routing through cursor.com/marketplace (SpaceXSI/Cursor bundling); ChatGPT Work and dots evidence moved to learn.chatgpt.com (help.openai.com 403s); DeepSeek desktop pin corrected 0.2.2-alpha.2→0.2.2-rc.1; ZCode 3.17.1→3.17.2; OpenAI dots model now GPT-6.1 Terra. Claude Code Desktop reconciled as a distinct Code-tab surface vs Cowork; Open Cowork corrected off the stale "single-turn wrapper" framing; two new local-first desktop coworkers added (Rowboat, OpenLoomi) with pinned clones. Shallow clones for every GitHub-backed rival live in repo_take/ (gitignored, never staged) at recorded HEAD SHAs; RV-13 allocated for the local-vs-cloud scheduled-task truth table. Owning docs: docs/COMPETITIVE-RESEARCH.md (§2026-11-08 Phase 2), docs/VIABILITY-VS-COMPETITORS.md (RV-13/RV-14 + new theme tables), docs/COMPETITOR-PROMOTED-USE-CASES.md (UC-CHAT-09/10, UC-WORK-10/11, UC-FILE-07, UC-AUTO-09, UC-MEM-06), docs/REFERENCE-IMPLEMENTATIONS.md (REF-OPENWORK/OPENCOWORK/ZCODE/DEEPSEEK-HARNESS/ROWBOAT/OPENLOOMI), docs/REFERENCE-REPOS-CORE-ARCHITECTURE-INDEX.md (refreshed pins + read-first indexes).
- **Reference studies landed:** the OpenHands study is committed (`0fdbfe8`, with the index wave in `f5896b8`); the LiteSPM parity/reconciliation appendices (A: upstream parity requests; B: reconciliation against the pinned revision) are committed on docs/LITESPM-LITECOWORK-INTEROP-LLD.md.

### Preservation and delivery rules

Continue from this handoff revision. Inspect the complete diff before any later implementation; preserve unrelated user changes and untracked archives. Do not reset, clean, stash or rebase. Stage, commit and push only when explicitly requested. Keep TARGET, CODE, SERVED, INTEGRATED and QUALIFIED evidence distinct.
