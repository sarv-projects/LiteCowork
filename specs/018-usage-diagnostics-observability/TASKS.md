# Tasks - Usage, Diagnostics and Observability

**Feature:** 018 | **Status:** IN_REVIEW - pending owner acceptance of SPEC and B-42; implementation not authorized; tasks not started unless marked.
**Spec revision:** 0.1 | **Release target:** V1 (release matrix rows: Windows x64 + OpenCode v2.0.32 first; Linux x64 second)
**Owners:** Observability, Operator/API, Desktop UI, Security/Trust, Agent Runtime, Product (see SPEC.md). Delivery wave: Wave 6 (gate G); the normalization contract (T03) is needed earlier, by Feature 001's usage write task in Wave 1.

## Current baseline

Verified against the checkout on branch `docs/audit-competitor-superset` on 2026-11-09.

- **Usage:** `grep -rn "usage_records\|UsageRecord\|UsageSummary\|QuotaSnapshot"` over `apps`, `crates` and `docs/schemas` finds only the generated type `ConversationUsageSummary` in `apps/litecowork-ui/src/api/operator-api.ts`. No table, writer, reader or route exists. `apps/litecoworkd/src/agents/opencode_server.rs` parses no token or cost field. OpenAPI declares `getConversationUsage` and `listTaskUsage` with schemas `ConversationUsageSummary`, `ConversationCurrencyCost`, `ConversationAgentUsage`, `UsageObservation`; both are `target_only` in `docs/schemas/served-route-inventory.json`.
- **Quota:** `apps/litecoworkd/src/agent_quota_observation_operator.rs` serves `GET /v1/agent-bindings/{agent_binding_id}/quota-observation` (owner check, `no_store_middleware`) and always returns `null`; tests `registered_route_returns_uncached_null_for_owned_binding`, `registered_route_denies_foreign_workspace_owner_without_cache`, `quota_observation_route_hides_bindings_outside_selected_workspace`. Tauri `apps/litecowork-ui/src-tauri/src/agent_quota_bridge.rs` and UI `apps/litecowork-ui/src/agents/QuotaObservation.tsx` render null as "No qualified usage observation."
- **Diagnostics:** 12 `eprintln!` calls (`main.rs` 3, `operator.rs` 4, `conversation_operator.rs` 2, `agents/supervisor.rs` 1, `backup_operator.rs` 1, `runtime_windows_security_tests.rs` 1). No `tracing` direct dependency in `apps/litecoworkd/Cargo.toml`, no log file, no ring buffer, no `panic::set_hook` in the daemon or `apps/litecowork-ui/src-tauri`. `GET /v1/runtime-lifecycle` is served from `operator.rs`; `litecoworkd --version` is covered by `version_matches_the_built_package` in `apps/litecoworkd/tests/cli.rs`.
- **UI:** `apps/litecowork-ui/src/settings/SettingsView.tsx` `SettingsCategory` = `general | workspaces | agents | runtime`; no Usage or About panel.

Gaps to V1: normalization contract, OpenCode counter extraction, reader, routes, structured log, crash records, health projection, support bundle, two Settings panels, and CODE/SYSTEM/USER qualification. Cross-package: Feature 001 B-10 write path (its usage record task) is NOT STARTED, so the dashboard has no data until it lands.

## 018-T01 - Owner acceptance of SPEC and blockers

- **Status:** BLOCKED - B-42 (support-bundle contents and redaction allowlist) is PROPOSED, pending the repository owner (sarv-projects). The package is IN_REVIEW.
- **Depends on:** NONE
- **Cross-package prerequisites:** NONE (orchestrator records B-42 in docs/CONTRADICTION-AND-ALIGNMENT-REGISTER.md).
- **Owner:** Product (repository owner), Observability, Security.
- **Allowed paths:** `specs/018-usage-diagnostics-observability/` only.
- **Contract:** SPEC sections 2, 3, 5 (B-42 and vocabulary mapping); AC-001 to AC-021.
- **Change intent:** owner reviews the allowlist, exclusions, local-only crash rule, defaults in SPEC section 12 and the vocabulary mapping; on acceptance flip SPEC and WIRING to ACCEPTED/LOCKED, revision 1.0, mark B-42 ACCEPTED with any rider.
- **Acceptance:**
  - [ ] Owner decision on B-42 and on the default caps recorded with date.
  - [ ] SPEC and WIRING status match and the validator passes.
  - [ ] Negative: no task below starts while this task is BLOCKED.
- **Command:** `python3 .specify/scripts/validate_feature_package.py`
- **Evidence:** register row B-42 and SPEC section 5.
- **Stop:** owner widens the allowlist to include content; re-run the privacy review before any task starts.

## 018-T02 - Promote accepted interfaces and the vocabulary mapping

- **Status:** NOT STARTED - usage routes are target-only; no diagnostics routes or error codes exist; `attribution_source` (Feature 001) and `UsageConfidence` (LLD) are not mapped in any owner doc.
- **Depends on:** T01
- **Cross-package prerequisites:** Feature 001 contract promotion task (the usage record shape is promoted there; this task must review it in the same change).
- **Owner:** Observability, API, Schemas owners.
- **Allowed paths:** `docs/OBSERVABILITY.md`, `docs/V1-TELEMETRY-USAGE-RESILIENCE-LLD.md`, `docs/API.md`, `docs/SCHEMAS.md`, `docs/schemas/operator-api.openapi.yaml`, `docs/schemas/error-codes.schema.json`, this package.
- **Contract:** SPEC sections 5 (mapping, B-42), 7.2; AC-001, AC-008.
- **Change intent:** promote `getWorkspaceUsageSummary`, `listWorkspaceUsageItems`, `exportWorkspaceUsageReport`, `getBuildInfo`, `listHealthProbes`, `listDiagnosticLogEntries`, `listCrashRecords`, `previewSupportBundle`, `writeSupportBundle`; record the PROVIDER_REPORTED -> NATIVE_REPORTED and ESTIMATED_LOCAL -> TOKENIZER_ESTIMATE mapping and the "turn-level observation, inference_count null unless verified" rule in the LLD; keep `QuotaObservation` as the only quota wire schema (O-02); add errors `USAGE_WINDOW_INVALID`, `SUPPORT_BUNDLE_TOO_LARGE`, `DIAGNOSTICS_UNAVAILABLE`. Label each new name "accepted target interface (not yet in source)".
- **Acceptance:**
  - [ ] Every new name exists once in its owner doc and machine schema.
  - [ ] No second quota schema is introduced.
  - [ ] Negative: schema rejects a usage item carrying a `content` or `prompt` field.
- **Command:** `python3 .specify/scripts/validate_feature_package.py` and `python3 scripts/validate_architecture.py`
- **Evidence:** PR link and output in Completion evidence.
- **Stop:** Feature 001's promoted usage record shape conflicts with the mapping; resolve with 001 before either merges.

## 018-T03 - Usage normalization contract for Feature 001 settlement

- **Status:** NOT STARTED - no `usage.rs` in `crates/storage-core/src`.
- **Depends on:** T01, T02
- **Cross-package prerequisites:** Feature 001 usage write task consumes `normalize_turn_usage` inside its settlement transaction.
- **Owner:** Observability, Storage.
- **Allowed paths:** `crates/storage-core/src/usage.rs` (new), `crates/storage-core/src/lib.rs`.
- **Contract:** SPEC 7.2 normalization, mapping, L-07; AC-001, AC-003, AC-005.
- **Change intent:** add `UsageAttribution`, `UsageCompleteness`, `AdapterUsageCounters`, `TurnUsageDraft`, `normalize_turn_usage`, `USAGE_MAX_TOKEN_COUNT`; treat cache and reasoning counters as subsets (never added to totals); out-of-range values become UNKNOWN; pure functions with no I/O so 001 can call them inside its transaction.
- **Acceptance:**
  - [ ] Reported, estimated and unknown inputs map as in SPEC section 5 (AC-001).
  - [ ] Negative, overflow and missing counters become UNKNOWN with PARTIAL completeness (AC-003).
  - [ ] Same `provider_response_id` yields the same draft identity for replay dedupe (AC-005).
  - [ ] Draft type has no string field able to hold message content (compile-time review plus test).
- **Command:** `cargo test -p storage-core --locked usage` (filter to be added by this task)
- **Evidence:** test output in Completion evidence.
- **Stop:** 001 needs a field this contract does not define; extend here, not in 001.

## 018-T04 - Extract typed usage counters from the OpenCode v2.0.32 adapter

- **Status:** NOT STARTED - `opencode_server.rs` parses no token, cost or usage field; O-01 is open.
- **Depends on:** T01, T03
- **Cross-package prerequisites:** Feature 001 OpenCode adapter qualification task (same file; coordinate the change in one review).
- **Owner:** Agent Runtime, Observability.
- **Allowed paths:** `apps/litecoworkd/src/agents/opencode_server.rs`, `apps/litecoworkd/src/agents/supervisor.rs` (pass-through of counters only), adjacent adapter tests.
- **Contract:** SPEC O-01, trust boundary 1; AC-001, AC-005.
- **Change intent:** record a pinned OpenCode v2.0.32 fixture of the final message/session event; if it carries token counters, parse only the typed numeric fields, model id and response id into `AdapterUsageCounters` with bounds; drop every text field; if it carries none, keep `None` so normalization yields ESTIMATED_LOCAL and record the finding in RESEARCH.md.
- **Acceptance:**
  - [ ] Fixture-based test parses counters or proves absence (AC-001).
  - [ ] Malformed or oversized counter values become `None` for that field (AC-003 behavior via T03).
  - [ ] Canary: message text in the fixture never reaches `AdapterUsageCounters`.
- **Command:** `cargo test -p litecoworkd --locked opencode_usage` (filter to be added by this task)
- **Evidence:** fixture path and test output; RESEARCH.md O-01 result.
- **Stop:** counters are only available by reading native session files; do not add a reader in V1 without O-04 approval.

## 018-T05 - Usage ledger reader and projections

- **Status:** NOT STARTED - no reader exists; depends on Feature 001's usage table.
- **Depends on:** T01, T03
- **Cross-package prerequisites:** Feature 001 usage write task (table and rows exist).
- **Owner:** Observability, Storage.
- **Allowed paths:** `crates/storage-core/src/usage.rs`, `crates/storage-sqlite/src/usage.rs` (new), `crates/storage-sqlite/src/usage_tests.rs` (new), `crates/storage-sqlite/src/lib.rs`, `docs/schemas/sqlite-vNN.sql` (new, only for a covering index if Feature 001's migration lacks it).
- **Contract:** SPEC 7.2 reader; L-01, L-02; AC-004, AC-006, AC-007, AC-021.
- **Change intent:** implement `UsageLedgerReader` (`workspace_usage_summary`, `workspace_usage_items`, `conversation_usage`) as SQL `GROUP BY` over leaf records with explicit bucket boundaries supplied by the caller; separate lanes for measured (NATIVE_REPORTED), estimated (TOKENIZER_ESTIMATE) and unknown; costs by currency with nullable decimal strings; `inference_count` null unless response ids are present; owner and Workspace predicate in every query.
- **Acceptance:**
  - [ ] Estimated rows never enter measured totals or coverage (AC-004).
  - [ ] Two currencies stay separate; no price source gives null cost (AC-006).
  - [ ] DST and year-boundary windows return each record exactly once (AC-007).
  - [ ] Conversation with two bindings sums each record once with per-binding drill-down (AC-021).
  - [ ] Foreign principal returns not found (AC-008).
- **Command:** `cargo test -p storage-sqlite --locked usage`
- **Evidence:** test output in Completion evidence.
- **Stop:** a rollup needs another rollup as input; redesign as leaf query.

## 018-T06 - Usage Operator routes and usage export

- **Status:** NOT STARTED - `getConversationUsage` is target-only; no Workspace usage route.
- **Depends on:** T01, T02, T05
- **Cross-package prerequisites:** Feature 011 authenticated IPC on Windows named pipe and Linux socket.
- **Owner:** Operator/API, Observability.
- **Allowed paths:** `apps/litecoworkd/src/usage_operator.rs` (new), `apps/litecoworkd/src/usage_operator_tests.rs` (new), `apps/litecoworkd/src/operator.rs` (router mount only).
- **Contract:** SPEC 7.2 route table; AC-008, AC-017, AC-021.
- **Change intent:** mount `getConversationUsage`, `getWorkspaceUsageSummary`, `listWorkspaceUsageItems`, `exportWorkspaceUsageReport` behind `super::authenticate`, `ensure_workspace_owner_for` and `no-store`; validate window (start before end, at most 400 buckets, RFC 3339 UTC) with `USAGE_WINDOW_INVALID`; export writes a bounded JSON-lines file of leaf metadata to an owner-chosen destination handle; run queries in `spawn_blocking`.
- **Acceptance:**
  - [ ] Owner gets the summary; foreign principal and missing scope selector are denied (AC-008).
  - [ ] Export contains only counts, ids, attribution and timestamps and respects the cap (AC-017).
  - [ ] Conversation usage across two bindings matches the reader (AC-021).
  - [ ] Served-route inventory lists the new routes.
- **Command:** `cargo test -p litecoworkd --locked usage_operator` (filter to be added by this task) and `python3 scripts/audit_served_routes.py`
- **Evidence:** test output and inventory diff.
- **Stop:** NONE beyond test failure.

## 018-T07 - Structured redacted operational log replacing eprintln

- **Status:** NOT STARTED - 12 `eprintln!` sites; no log sink.
- **Depends on:** T01, T02
- **Cross-package prerequisites:** Feature 001 operator header and dispatch-admission task touches the same `eprintln!` sites in `conversation_operator.rs`; coordinate.
- **Owner:** Observability, Security.
- **Allowed paths:** `apps/litecoworkd/src/diagnostics/mod.rs` (new), `apps/litecoworkd/src/diagnostics/log.rs` (new), `apps/litecoworkd/src/diagnostics/log_tests.rs` (new), `apps/litecoworkd/src/main.rs`, `apps/litecoworkd/src/operator.rs`, `apps/litecoworkd/src/conversation_operator.rs`, `apps/litecoworkd/src/agents/supervisor.rs`, `apps/litecoworkd/src/backup_operator.rs` (log call sites only), `apps/litecoworkd/Cargo.toml` (only if F-01 selects `tracing`).
- **Contract:** SPEC B-42 category 4, F-01, section 10; AC-011, AC-015.
- **Change intent:** add `DiagnosticEntry` (no free-form text), `DiagnosticLog` bounded ring plus background rotating JSON-lines writer under `<data-dir>/diagnostics/log/` with owner-only directory permissions; replace each daemon `eprintln!` with a typed event code; keep the CLI's one-JSON-document stdout/stderr contract in `backup_operator.rs`; bounded channel drops debug entries and counts drops.
- **Acceptance:**
  - [ ] Canary prompt, completion, token, cookie, handle and path never appear in ring or files (AC-011).
  - [ ] Blocked writer does not change turn admission latency beyond noise; dropped counter increments (AC-015).
  - [ ] Rotation keeps files within the cap across restart (AC-018).
  - [ ] No non-test `eprintln!` remains outside CLI output paths (grep check).
- **Command:** `cargo test -p litecoworkd --locked diagnostics_log` (filter to be added by this task) and `cargo clippy -p litecoworkd --locked -- -D warnings`
- **Evidence:** test output and grep result.
- **Stop:** a call site needs user content to be useful; log an id and a code instead.

## 018-T08 - Local crash records for daemon and desktop

- **Status:** NOT STARTED - no panic hook in `apps/litecoworkd` or `apps/litecowork-ui/src-tauri`.
- **Depends on:** T01, T07
- **Cross-package prerequisites:** NONE
- **Owner:** Observability, Desktop (Tauri).
- **Allowed paths:** `apps/litecoworkd/src/diagnostics/crash.rs` (new), `apps/litecoworkd/src/main.rs`, `apps/litecowork-ui/src-tauri/src/diagnostics_bridge.rs` (new), `apps/litecowork-ui/src-tauri/src/lib.rs`, `apps/litecowork-ui/src-tauri/src/main.rs`.
- **Contract:** SPEC B-42 category 5 (crash reports stay local); AC-013, AC-018.
- **Change intent:** `install_crash_hook` chained before the default hook writes a bounded JSON record (build, OS, time, LiteCowork file and line, symbol-only frames, redacted class) to `<data-dir>/diagnostics/crashes/`, keeping the newest 20; the Tauri process installs the same hook into the app data directory; a WebView error boundary reports only error name and component name through `record_desktop_crash`; no network dependency in the module.
- **Acceptance:**
  - [ ] Injected panic produces one bounded record; payload text beyond the bound is dropped (AC-013).
  - [ ] Record from the previous incarnation is listed after restart (AC-018).
  - [ ] Test asserts the diagnostics module does not reference `reqwest` (AC-013).
- **Command:** `cargo test -p litecoworkd --locked diagnostics_crash` (filter to be added by this task) and `cargo test -p litecowork-ui --locked diagnostics_bridge` (filter to be added by this task)
- **Evidence:** test output in Completion evidence.
- **Stop:** the hook cannot write without allocating unbounded memory; reduce the record.

## 018-T09 - Health probe projection and build info

- **Status:** PARTIAL - `GET /v1/runtime-lifecycle` and `litecoworkd status` exist; no per-probe projection with age and no build-info route.
- **Depends on:** T01, T02
- **Cross-package prerequisites:** Feature 011 Runtime readiness and startup-blocker model.
- **Owner:** Observability, Runtime Lifecycle.
- **Allowed paths:** `apps/litecoworkd/src/diagnostics/health.rs` (new), `apps/litecoworkd/src/diagnostics_operator.rs` (new), `apps/litecoworkd/src/operator.rs` (router mount only).
- **Contract:** SPEC G-08, 7.2; AC-010, AC-014.
- **Change intent:** `listHealthProbes` reports Operator serving, IPC, SQLite writer, startup blockers, AgentModule discovery and quota producer presence as READY, DEGRADED, BLOCKED, UNKNOWN or OFFLINE with `observed_at` and `stale_after_ms`; single-flight cache for slow probes; `getBuildInfo` returns `CARGO_PKG_VERSION`, schema version, OS family/version and architecture.
- **Acceptance:**
  - [ ] Serving is reported separately from execution readiness (AC-014).
  - [ ] A stale probe shows its age and UNKNOWN after the cutoff (AC-014).
  - [ ] Build info matches the built package version (AC-010).
- **Command:** `cargo test -p litecoworkd --locked diagnostics_health` (filter to be added by this task)
- **Evidence:** test output in Completion evidence.
- **Stop:** a probe would mutate state or call a provider; drop it.

## 018-T10 - Support bundle preview and write (B-42)

- **Status:** NOT STARTED.
- **Depends on:** T01, T07, T08, T09
- **Cross-package prerequisites:** Feature 019 secret scan (`scripts/v1_secret_scan.py`) runs over produced bundles in CI.
- **Owner:** Observability, Security.
- **Allowed paths:** `apps/litecoworkd/src/diagnostics/bundle.rs` (new), `apps/litecoworkd/src/diagnostics/bundle_tests.rs` (new), `apps/litecoworkd/src/diagnostics_operator.rs`.
- **Contract:** SPEC B-42; AC-012.
- **Change intent:** `preview_support_bundle` lists categories, item counts and field classes; `write_support_bundle` writes only selected allowlisted categories with home-prefix replacement, a manifest of per-file SHA-256 and a 10 MiB cap (`SUPPORT_BUNDLE_TOO_LARGE`), create-new semantics and fsync; mount `previewSupportBundle` and `writeSupportBundle` (Idempotency-Key).
- **Acceptance:**
  - [ ] Canary secret, prompt, Resource path and native handle seeded across the installation never appear in any bundle (AC-012).
  - [ ] Deselected categories are absent; manifest digests verify (AC-012).
  - [ ] Over-cap selection fails before writing (AC-012).
  - [ ] No network call (module dependency test) (AC-012).
- **Command:** `cargo test -p litecoworkd --locked diagnostics_bundle` (filter to be added by this task) and `python3 scripts/v1_secret_scan.py`
- **Evidence:** test output; secret scan output on a produced bundle.
- **Stop:** a category cannot be made content-free; remove it from the allowlist and record that in SPEC.

## 018-T11 - Tauri usage and diagnostics bridges

- **Status:** NOT STARTED - only `agent_quota_bridge.rs` exists.
- **Depends on:** T01, T06, T09, T10
- **Cross-package prerequisites:** NONE
- **Owner:** Desktop (Tauri).
- **Allowed paths:** `apps/litecowork-ui/src-tauri/src/usage_bridge.rs` (new), `apps/litecowork-ui/src-tauri/src/diagnostics_bridge.rs`, `apps/litecowork-ui/src-tauri/src/lib.rs`, `apps/litecowork-ui/src/settings/usage/usage-api.ts` (new), `apps/litecowork-ui/src/settings/about/diagnostics-api.ts` (new).
- **Contract:** SPEC 7.2 Tauri commands; AC-016.
- **Change intent:** add `get_workspace_usage_summary`, `list_workspace_usage_items`, `get_conversation_usage`, `export_workspace_usage_report`, `get_build_info`, `list_health_probes`, `list_diagnostic_log_entries`, `list_crash_records`, `preview_support_bundle`, `write_support_bundle`; native save dialog for export and bundle destinations; decode with the same strictness as `decode_observation` (reject cross-scope data).
- **Acceptance:**
  - [ ] Each command rejects responses for another Workspace or binding.
  - [ ] Unknown error codes map to a recoverable state, never success.
- **Command:** `cargo test -p litecowork-ui --locked usage_bridge` (filter to be added by this task) and `pnpm --dir apps/litecowork-ui run build`
- **Evidence:** test output in Completion evidence.
- **Stop:** NONE beyond test failure.

## 018-T12 - Settings "Usage & Monitoring" and "About & Diagnostics" UI

- **Status:** NOT STARTED - neither category exists in `SettingsView.tsx`; `QuotaObservation.tsx` is the only usage-related component.
- **Depends on:** T01, T11
- **Cross-package prerequisites:** Feature 011 Settings shell category registration and keyboard baseline; Feature 002 Agent profile "Usage shortcut" opens this panel with the binding filter.
- **Owner:** Desktop UI.
- **Allowed paths:** `apps/litecowork-ui/src/settings/SettingsView.tsx` (category entries only), `apps/litecowork-ui/src/settings/usage/UsageMonitoringPanel.tsx` (new), `apps/litecowork-ui/src/settings/usage/usage-state.ts` (new), `apps/litecowork-ui/src/settings/usage/usage-window.ts` (new), `apps/litecowork-ui/src/settings/about/AboutDiagnosticsPanel.tsx` (new), `apps/litecowork-ui/src/agents/QuotaObservation.tsx` (reuse only), `apps/litecowork-ui/tests/usage-state.test.ts` (new), `apps/litecowork-ui/tests/usage-window.test.ts` (new), `apps/litecowork-ui/tests/usage-honesty.test.mjs` (new), `apps/litecowork-ui/tests/about-diagnostics.test.mjs` (new).
- **Contract:** SPEC AC-002, AC-004, AC-009, AC-010, AC-013, AC-016, AC-020; contracts/ui-behavior.md; V1-SETTINGS-CONTRACT-LEDGER rows `inspectUsageTotals` to `exportUsageReport` and `showVersion` to `openSupportDocs`.
- **Change intent:** Usage panel with quick ranges, Workspace scope, binding filter, headline metrics with coverage and source, "(est.)" lane, "Not priced" cost lanes, quota cards via `QuotaObservationSummary` ("Not reported by this agent" for null), accessible sortable table and drill-down to leaf metadata, Export; `usage-window.ts` computes half-open UTC boundaries from `Intl.DateTimeFormat().resolvedOptions().timeZone`; About panel with build info and Copy, health probes with age, recent log entries, crash records, support bundle preview and write, Run provider diagnostic shown unavailable with reason; money displayed from decimal strings, tokens checked with `Number.isSafeInteger`.
- **Acceptance:**
  - [ ] Empty state text matches SPEC and the literals `$0.00`, `0 tokens` and `100%` never render by default (AC-002).
  - [ ] Estimated values carry "(est.)" and are excluded from measured headline numbers (AC-004).
  - [ ] Null quota shows "Not reported by this agent"; expired shows Unknown with reason (AC-009).
  - [ ] Copy build info copies exactly the displayed fields (AC-010).
  - [ ] Crash records from a previous run are listed (AC-013).
  - [ ] Every chart has a table; keyboard and screen reader checks pass (AC-016).
  - [ ] Provider diagnostic is unavailable with reason (AC-020).
- **Command:** `pnpm --dir apps/litecowork-ui test` and `pnpm --dir apps/litecowork-ui run build`
- **Evidence:** test output; Settings ledger rows updated by T17.
- **Stop:** a control has no served route on a row; omit it.

## 018-T13 - Failure, recovery and backpressure suite with the canonical test target

- **Status:** NOT STARTED - `apps/litecoworkd/tests/usage_diagnostics.rs` does not exist.
- **Depends on:** T01, T05-T10
- **Cross-package prerequisites:** Feature 001 settlement and replay fixtures.
- **Owner:** Observability, Operator/API.
- **Allowed paths:** `apps/litecoworkd/tests/usage_diagnostics.rs` (new).
- **Contract:** SPEC section 11, contracts/errors-recovery.md; AC-015, AC-018.
- **Change intent:** spawn the real daemon: settle turns with and without counters, replay final commits, restart and compare summaries; block the log writer and measure admission latency; panic a worker thread and check the crash record after restart; kill during bundle write and confirm no partial bundle is reported.
- **Acceptance:**
  - [ ] Summary identical before and after restart (AC-018).
  - [ ] Blocked log sink does not affect settlement (AC-015).
  - [ ] Replay adds no usage (AC-005).
  - [ ] Interrupted bundle write leaves no advertised file.
- **Command:** `cargo test -p litecoworkd --test usage_diagnostics --locked -- --nocapture`
- **Evidence:** test output per OS.
- **Stop:** telemetry failure blocks settlement; fix before anything else.

## 018-T14 - Usage query performance baseline

- **Status:** NOT STARTED - no baseline; LLD forbids hard-coded SLOs before profiling.
- **Depends on:** T01, T05, T13
- **Cross-package prerequisites:** Feature 019 soak/performance harness (`scripts/v1_soak_harness.py`).
- **Owner:** Observability, Release.
- **Allowed paths:** `apps/litecoworkd/tests/usage_diagnostics.rs`, `crates/storage-sqlite/src/usage_tests.rs`, this package RESEARCH.md.
- **Contract:** SPEC AC-019; O-05.
- **Change intent:** generate the reference fixture (record count in RESEARCH), assert `EXPLAIN QUERY PLAN` uses the covering index, record summary and items latency per OS, then propose thresholds for owner review.
- **Acceptance:**
  - [ ] Query plan uses the index for summary and items (AC-019).
  - [ ] Baseline recorded with OS, build and hardware.
- **Command:** `cargo test -p storage-sqlite --locked usage_query_plan` (filter to be added by this task)
- **Evidence:** RESEARCH.md baseline table; CURRENT-RUN.md entry.
- **Stop:** full table scan on the summary query.

## 018-T15 - Cross-layer trace for each user-visible operation

- **Status:** NOT STARTED.
- **Depends on:** T01, T06, T09-T12
- **Cross-package prerequisites:** NONE
- **Owner:** Observability, Desktop UI.
- **Allowed paths:** `specs/018-usage-diagnostics-observability/contracts/ui-behavior.md`, `specs/018-usage-diagnostics-observability/WIRING.yaml`.
- **Contract:** AGENTS.md section 4; AC-002, AC-012.
- **Change intent:** for View usage, Filter, Drill down, Export usage, View quota, Copy build info, View health, View logs, View crash records, Preview bundle, Write bundle record: user outcome, exact control in `UsageMonitoringPanel.tsx` / `AboutDiagnosticsPanel.tsx`, Tauri command, Operator route, authorization, service owner, read/write boundary, record or file, projection, restart behavior and evidence test.
- **Acceptance:**
  - [ ] Every operation has a complete row; no route without a UI path and no control without a route.
  - [ ] WIRING edges marked served only with T16 evidence.
- **Command:** `python3 .specify/scripts/validate_feature_package.py`
- **Evidence:** updated contracts/ui-behavior.md.
- **Stop:** a hop is missing; open a task.

## 018-T16 - Qualify on the release matrix rows

- **Status:** NOT STARTED.
- **Depends on:** T01-T15
- **Cross-package prerequisites:** Feature 001 native Send qualified on the row (usage needs real turns); Feature 019 release matrix rows and evidence bundle; Feature 011 packaged sidecar and IPC.
- **Owner:** Product (USER sign-off), Observability, Security, Release.
- **Allowed paths:** this package; `docs/IMPLEMENTATION-STATUS.md` evidence rows (via T17).
- **Contract:** ACCEPTANCE-AND-E2E-PLAN E2E-01 (usage after a real Send) and the diagnostics journey the orchestrator adds for O-03; AC-001, AC-002, AC-012, AC-013.
- **Change intent:** on the packaged desktop for Windows x64 + OpenCode v2.0.32 and Linux x64: empty usage state before any Send; real Sends and the resulting labelled usage; restart and unchanged summary; forced daemon panic and crash record after restart; support bundle preview, deselection and write, then secret scan over the bundle; denial with a foreign Workspace.
- **Acceptance:**
  - [ ] CODE: canonical command passes on each row.
  - [ ] SYSTEM: packaged run log per row.
  - [ ] USER: owner sign-off per row including denial and recovery.
  - [ ] Secret scan finds nothing in the produced bundle (AC-012).
- **Command:** `cargo test -p storage-sqlite --locked usage && cargo test -p litecoworkd --test usage_diagnostics --locked -- --nocapture && pnpm --dir apps/litecowork-ui test`
- **Evidence:** evidence bundle path per row.
- **Stop:** a row lacks USER evidence; that row stays unqualified and the panels show unavailable there.

## 018-T17 - Documentation and inventory sync

- **Status:** NOT STARTED.
- **Depends on:** T01, T15, T16
- **Cross-package prerequisites:** NONE (the orchestrator owns shared ledgers; this task supplies the edits for review).
- **Owner:** Observability, Documentation owners.
- **Allowed paths:** `docs/IMPLEMENTATION-STATUS.md`, `docs/schemas/served-route-inventory.json`, `docs/V1-SETTINGS-CONTRACT-LEDGER.md`, `docs/V1-UI-ACTION-STATE-LEDGER.md`, `docs/V1-UI-SCREEN-CONTRACT.md`, `docs/CONTRADICTION-AND-ALIGNMENT-REGISTER.md`, `CURRENT-RUN.md`, this package.
- **Contract:** AGENTS.md sections 4-5; SPEC section 4; AC-008.
- **Change intent:** regenerate the served-route inventory; update IMPLEMENTATION-STATUS "Usage and telemetry" per row and evidence class; update Settings ledger Usage & Monitoring and About/Diagnostics rows; close B-42; refresh SPEC section 4.
- **Acceptance:**
  - [ ] `python3 scripts/audit_served_routes.py` matches the committed inventory.
  - [ ] No row claims QUALIFIED without the T16 bundle for that row.
- **Command:** `python3 scripts/audit_served_routes.py` and `python3 .specify/scripts/validate_feature_package.py`
- **Evidence:** diff of status documents.
- **Stop:** evidence and claim disagree; downgrade the claim.

## 018-T18 - Package VERIFIED for V1

- **Status:** NOT STARTED.
- **Depends on:** T01-T17
- **Cross-package prerequisites:** Feature 019 evidence bundle process and release qualification matrix.
- **Owner:** Product (sign-off), Observability, Security.
- **Allowed paths:** this package; `docs/IMPLEMENTATION-STATUS.md`.
- **Contract:** SPEC sections 14-15; AC-001 to AC-021; SPECIFICATION-STANDARD lifecycle.
- **Change intent:** assemble the evidence bundle; move ACCEPTED/LOCKED -> IMPLEMENTING at the first implementation merge and IMPLEMENTING -> VERIFIED after owner sign-off on both rows.
- **Acceptance:**
  - [ ] Canonical command passes on both rows.
  - [ ] Owner USER sign-off per row including denial and recovery.
  - [ ] SPEC and WIRING status equal VERIFIED and the validator passes.
- **Command:** `cargo test -p storage-sqlite --locked usage && cargo test -p litecoworkd --test usage_diagnostics --locked -- --nocapture && pnpm --dir apps/litecowork-ui test` and `python3 .specify/scripts/validate_feature_package.py`
- **Evidence:** evidence bundle path referenced from IMPLEMENTATION-STATUS.md.
- **Stop:** any row lacks USER evidence.

## Implementation and optimization notes (Rust and TypeScript)

Findings from reading the source, folded into the tasks above. They add no scope.

- **Logs by construction, not by filter (Rust, T07):** a regex redaction pass over free-form log strings misses new formats. A `DiagnosticEntry` with only `&'static str` codes and typed ids makes leaking content a compile error rather than a review finding. `tracing` is already in `Cargo.lock` transitively, so F-01 can use it with a field allowlist layer at no new supply-chain cost, or a 100-line in-house ring is enough for V1.
- **Aggregate in SQLite, format in TypeScript (Rust/TS, T05/T12):** rollups belong in one `GROUP BY` over leaves with a `(workspace_id, observed_at)` covering index. The WebView only formats. Money stays a decimal string end to end; never parse it into a JS `number`.
- **Timezone without a tz database in Rust (Rust/TS, T05/T12):** the `time` crate has no IANA zone data. The UI already has `Intl`; it can compute half-open UTC bucket boundaries for the viewer's zone and send them, and the daemon groups by those boundaries. That avoids adding `chrono-tz` and keeps DST correctness testable in `node --test`.
- **Reuse the honest quota component (TS, T12):** `QuotaObservation.tsx` already distinguishes null, expired and future-dated observations. The dashboard should reuse `QuotaObservationSummary` instead of a second renderer.
- **Shared no-store middleware (Rust, T06/T09):** `agent_quota_observation_operator.rs` defines its own `no_store_middleware`; the new usage and diagnostics routers should share one helper rather than copying it again.
- **Panic hook cost (Rust, T08):** the hook must not format the panic payload into an unbounded `String`; write a fixed-shape record with a 512-byte bound to keep it safe inside a panicking process.
- **Keep authority in Rust (all):** normalization, redaction, bundle assembly and caps run in the daemon. The WebView never decides what is safe to export.

## Dependency order

T01 -> T02 -> T03 -> (T04, T05 in parallel) -> T06 (after T05). T02 -> T07 -> T08. T02 -> T09. T10 after T07, T08, T09. T11 after T06, T09, T10 -> T12. T13 after T05-T10 -> T14. T15 after T06, T09-T12. T16 after T01-T15 -> T17 -> T18. Every task depends on T01.

## Cross-package dependencies

| This task | Other package task | Reason |
|---|---|---|
| T02 | Feature 001 contract promotion | Usage record shape and mapping are promoted together |
| T03, T05, T13 | Feature 001 usage write at settlement (B-10) | 001 calls the normalizer and writes the rows this package reads |
| T04 | Feature 001 OpenCode adapter qualification | Same adapter file; counters are parsed from the same final event |
| T07 | Feature 001 operator header and dispatch-admission cleanup | Same `eprintln!` sites in `conversation_operator.rs` |
| T06, T09, T16 | Feature 011 IPC, Runtime readiness, packaged sidecar | Routes and probes depend on the Runtime foundation |
| T12 | Feature 011 Settings shell; Feature 002 Agent profile Usage shortcut | Category registration and entry from the Agent profile |
| T10, T14, T16, T18 | Feature 019 secret scan, soak harness, release matrix and evidence bundle | Shared release tooling |
| T16 | Feature 001 native Send qualified per row | Usage needs real turns |

Cross-package task IDs are named by outcome because other packages' task plans are being renumbered in the same planning pass; the orchestrator resolves them to IDs in the traceability index.

## Completion evidence

For each task append a row when it closes: task, code revision (SHA), exact command, result (pass/fail with counts), OS and build (Windows 11/10 x64 or Linux x64 distribution), harness and version (OpenCode v2.0.32), evidence class reached (CODE, SYSTEM, USER), report path and reviewer. Statuses above are source-audit classifications at this revision, not qualification.

| Task | Revision | Command | Result | OS/build | Harness/version | Class | Reviewer |
|---|---|---|---|---|---|---|---|
