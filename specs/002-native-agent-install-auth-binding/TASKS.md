# Tasks - Native Agent Install, Auth and Binding

**Feature:** 002 | **Status:** ACCEPTED/LOCKED - implementation authorized; see each task Status line. Windows rows, T15 release-matrix qualification and release/signing/packaging items are DEFERRED to the owner's laptop testing (standing owner instruction); LiteSPM-dependent parts are deferred likewise.
**Spec revision:** 1.0 | **Release target:** V1 (release matrix rows: Windows x64 + OpenCode v2.0.32 first; Linux x64 second)

## Current baseline

Verified against source on 2026-11-09. Served today: ACP Registry and local inventory (`apps/litecoworkd/src/agent_registry_operator.rs`, `list_agent_installations` in `apps/litecoworkd/src/operator.rs`), the Codex/OpenCode profile probe (`probe_local_agent_profile`, `probe_local_opencode_profile` in `operator.rs`; `apps/litecoworkd/src/agents/opencode_profile_probe.rs`, `codex_profile_probe.rs`) and AgentBinding list/get/create/enable/disable/default (`operator.rs`, `crates/storage-sqlite/src/lib.rs` `create_agent_binding_transaction`/`enable_agent_binding_transaction`). Mounted but fail-closed: `get_control_descriptor`/`refresh_control_descriptor` in `apps/litecoworkd/src/agent_control_operator.rs` return `503` via `unavailable_descriptor()`, and `submit_credential` in `apps/litecoworkd/src/agent_credentials_routes.rs` uses the stub `NoAgentDescriptorSource`, so it always returns `503`. Implemented but unwired: the descriptor model and validator (`apps/litecoworkd/src/agents/control_descriptor.rs`) and `AgentModuleRegistry` (`apps/litecoworkd/src/agents/module_registry.rs`, file-level `dead_code`). Desktop: `apps/litecowork-ui/src/agents/AgentCatalogSettings.tsx`, `AgentControlDescriptorPanel.tsx`, `desktop-agent-control-api.ts`, Tauri commands `probe_agent_profile`/`create_agent_binding`/`enable_agent_binding` in `apps/litecowork-ui/src-tauri/src/lib.rs` and `agent_control_bridge.rs`. Gaps: the OpenCode offer can never be bound (SPEC §4a: hard-coded `compatible: false`, `OPENCODE` vs `opencode` provider key, Windows executable resolution, probe/dispatch executable drift, version pin), so Feature 001 Send is unreachable from the UI; no descriptor producer; no native sign-in or auth observation (auth is always `UNKNOWN`); no fresh-descriptor enable gate; no Claude Code profile, descriptor or auth path; install/update not served.

## 002-T01 - OpenCode v2.0.32 profile produces a bindable, dispatch-consistent offer

- **Status:** CODE-COMPLETE 2026-11-10 (offer admission computed in `agents/native_provider.rs`, `NativeProviderKey`, shared executable resolver, OpenAPI `profile_admission_supported` boolean and generated types synced; probe, native_provider_key and opencode_profile tests pass). Open: Windows x64 and Linux x64 probe-to-binding transcripts need a live OpenCode 2.0.32 binary (not available in this sandbox), so AC-013 is not evidenced.
- **Depends on:** NONE
- **Cross-package prerequisites:** NONE. This task is the P1 prerequisite that Feature 001 (first native Conversation route, gate A) and Feature 004 consume.
- **Owner:** Agent Runtime (probe, supervisor provider identity), Storage (gate unchanged, tests), Desktop UI (catalog copy).
- **Allowed paths:** `apps/litecoworkd/src/operator.rs` (only `probe_local_opencode_profile`, `opencode_endpoint_capabilities`, `resolve_local_executable` and their tests), `apps/litecoworkd/src/agents/opencode_profile_probe.rs`, `apps/litecoworkd/src/agents/supervisor.rs` (only `execute_agent_turn` provider match, `find_opencode_executable`, profile lookup), `apps/litecoworkd/src/agents/mod.rs`, `apps/litecoworkd/src/agents/native_provider.rs` (new), `apps/litecoworkd/src/conversation_operator_tests.rs` (fixture provider key), `crates/storage-sqlite/src/agent_binding_tests.rs`, `crates/storage-sqlite/src/runtime_offer_tests.rs`, `docs/AGENT-CONTROL.md`, `docs/schemas/operator-api.openapi.yaml` (OpenCodeProfileProbe schema only), `apps/litecowork-ui/src/api/operator-api.ts` (regenerated only), `apps/litecowork-ui/src/agents/AgentCatalogSettings.tsx`, `apps/litecowork-ui/tests/agent-catalog-settings.test.mjs`.
- **Contract:** SPEC §4a, §5 (B-02 rider via Feature 001, B-11), §7.2 (`NativeProviderKey`, `opencode_offer_admission`), §10, AC-013, AC-002; docs/AGENT-CONTROL.md; docs/V1-AGENT-DISCOVERY-AND-CATALOG-LLD.md.
- **Change intent:** Make the probe publish a truthful compatible offer only for the exact accepted route, and make dispatch use the same identity. Concretely:
  1. In `probe_local_opencode_profile`, replace the literal `compatible: false` and the `"DEGRADED"` readiness with a computed `opencode_offer_admission(&probe)`: `compatible: true` and readiness `STARTABLE` only when `probe.protocol_version == Some("2.0.32")` (`OpenCodeServerProtocol::V2_0_32`), `probe.probe_readiness == Complete` and `probe.host_process_stopped`; otherwise keep `compatible: false` with readiness `DEGRADED`/`UNAVAILABLE` and a `reason` string in the offer constraints. 2.0.26 and 2.0.28 stay observation-only. Keep the 120 s offer expiry.
  2. Set `OpenCodeProfileProbe::profile_admission_supported` from the same predicate (Conversation admission only; Task admission stays unsupported and keeps its own flag in copy). Update the OpenCodeProfileProbe schema in `docs/schemas/operator-api.openapi.yaml` from constant `false` to boolean and regenerate `operator-api.ts` with `api:generate`.
  3. In `opencode_endpoint_capabilities()`, record the observed `protocol_version` and set `input.text: true` (the only input the supervisor path sends); every other capability stays `false`. Write the endpoint after the probe so capabilities reflect the observed version.
  4. Add `NativeProviderKey` (`OpenCode`, `ClaudeCode`, `Codex`) in `agents/native_provider.rs` with `from_profile_key` for the stored upper-case form; change `execute_agent_turn` to match on the enum, so a stored `OPENCODE` reaches the OpenCode arm and unknown keys still fail closed. Replace the per-turn `list_agent_profiles` scan with a keyed profile read.
  5. Replace `resolve_local_executable("opencode")` with one shared resolver in `agents/` that applies the `find_opencode_executable` Windows rules (`.exe`, `.cmd`, npm `%APPDATA%\npm`) and canonicalizes; the supervisor dispatches the `endpoint_ref` registered by `register_local_endpoint_binding` for the selected endpoint, re-validates it exists and is the same canonical path, and fails closed on drift.
  6. Do NOT change `has_fresh_compatible_endpoint` or `resolve_conversation_session_host_in_transaction`; the fix is a truthful offer, not a weaker gate.
  7. In `AgentCatalogSettings.tsx`, derive the OpenCode footnote and lead-eligibility copy (currently hard-coded "cannot be bound for use") from the fresh observation; Create binding stays disabled for an incompatible or expired offer.
- **Acceptance:**
  - [ ] AC-013: on Windows x86_64 and on Linux x86_64 with OpenCode 2.0.32 installed, probe then create a disabled binding succeeds; the stored provider key parses to `NativeProviderKey::OpenCode`; the dispatched executable equals the registered `endpoint_ref`.
  - [ ] Negative: OpenCode 2.0.28, a partial probe, an unobserved host stop, an expired offer and an unenrolled Runtime each leave `compatible=false` and binding create returns the existing incompatible-offer error with no row written.
  - [ ] Negative: an `endpoint_ref` whose canonical path changed between probe and dispatch fails the turn closed with no native spawn.
  - [ ] Concurrency: two simultaneous probes for the same profile produce one current offer and no duplicate endpoint binding.
  - [ ] AC-002: the probe still creates no AgentBinding by itself.
  - [ ] UI: the catalog shows "Compatible for Workspace binding" only for a fresh compatible offer; keyboard path to Create binding works; status text is not color-only.
- **Command:** `cargo test -p litecoworkd --locked opencode_profile` ; `cargo test -p litecoworkd --locked native_provider_key` (filter to be added by this task) ; `cargo test -p storage-sqlite --locked agent_binding_tests` ; `pnpm --dir apps/litecowork-ui run api:check` ; `pnpm --dir apps/litecowork-ui test`
- **Evidence:** test names above plus a Windows x64 and Linux x64 probe-to-binding transcript recorded in docs/IMPLEMENTATION-STATUS.md (OpenCode row) and CURRENT-RUN.md.
- **Stop:** stop if making the offer compatible would require weakening a storage gate, if 2.0.32 cannot be detected exactly on Windows, or if the dispatched executable cannot be bound to the probed identity.

## 002-T02 - Promote the accepted 002 interfaces into owner docs and machine schemas

- **Status:** PARTIAL (no migration was needed: the Runtime-local descriptor table already exists since migration 16, so no `sqlite-vNN.sql` was added; `EnableBindingWithDescriptor` is promoted as the optional `X-Agent-Descriptor-Revision` header on `enableAgentBinding` in the OpenAPI with `RESOURCE_STALE` for DescriptorStale; probe budget is in-process, see T04. Not done: auth-observation DDL and `beginAgentAuthentication` response, Claude Code route-matrix row, SPEC revision relabel, which belong to T07/T12).
- **Depends on:** NONE
- **Cross-package prerequisites:** Feature 011 IPC/route conventions (Wave 0); coordinate the migration number with the orchestrator because other packages also add `sqlite-vNN.sql` files.
- **Owner:** Agent Runtime, Security/Trust, API/schema owner.
- **Allowed paths:** `docs/AGENT-CONTROL.md`, `docs/AGENT-ACCOUNT-ROUTE-MATRIX.md`, `docs/V1-AGENT-DISCOVERY-AND-CATALOG-LLD.md`, `docs/API.md`, `docs/EVENTS.md`, `docs/SCHEMAS.md`, `docs/schemas/operator-api.openapi.yaml`, `docs/schemas/error-codes.schema.json`, `docs/schemas/domain-event.schema.json`, `docs/schemas/sqlite-vNN.sql` (new, next unused number; v28 is the highest present), `apps/litecowork-ui/src/api/operator-api.ts` (regenerated), this package.
- **Contract:** SPEC §5 (B-11, B-12), §7.2, §11, AC-004, AC-008, AC-012, AC-014, AC-016, AC-017; docs/SPECIFICATION-STANDARD.md §7.
- **Change intent:** Promote together, in one reviewed change: `EnableBindingWithDescriptor` (enable request carries `descriptor_revision`); `AgentControlDescriptorFreshness` (TTL 900 s, 3 probes per minute per module, module-overridable downward); the Runtime-local descriptor record and auth-observation record DDL; `AgentAuthObservation` response for `beginAgentAuthentication` and a read of the latest observation; typed errors `DescriptorStale`, `OfferIncompatible`, `NotLeadEligible`, `ReplayConflict`, `MethodNotDeclared` mapped to canonical error codes (reuse existing codes where they exist, add only missing ones); the Claude Code CLI row in AGENT-ACCOUNT-ROUTE-MATRIX as "second module, not qualified". No event is added unless the owner promotes a Runtime-local observation event.
- **Acceptance:**
  - [ ] Every accepted target signature in SPEC §7.2 has one normative home and matching OpenAPI/JSON Schema/SQL; generated TypeScript passes `api:check`.
  - [ ] Negative: the migration is additive and immutable; no existing table semantics change; a downgrade refuses new admission that needs the new fields.
  - [ ] No secret-bearing field appears in any schema (AC-009 design check).
  - [ ] SPEC §7.2 entries promoted are relabelled as promoted in a new SPEC revision with owner approval.
- **Command:** `python3 .specify/scripts/validate_feature_package.py` ; `python3 scripts/audit_v1_contracts.py` ; `pnpm --dir apps/litecowork-ui run api:check`
- **Evidence:** reviewed diff of owner docs + schemas; validator and contract audit output in CURRENT-RUN.md.
- **Stop:** stop if a field would need a second normative definition or if the migration number collides with another package's migration.

## 002-T03 - Wire AgentModuleRegistry with the OpenCode v2.0.32 manifest and conformance fixture

- **Status:** CODE-COMPLETE 2026-11-10 (OpenCode 2.0.32 manifest in `agents/modules/opencode.rs` with TTL 900 s and probe budget 3 per minute fields validated by the registry, registered through `default_module_registry()`, pinned fixture `apps/litecoworkd/tests/fixtures/agent-modules/opencode-2.0.32.json`; the registry file keeps a reduced `dead_code` allow for the F13 lookups no route calls yet).
- **Depends on:** T02
- **Cross-package prerequisites:** NONE
- **Owner:** Agent Runtime.
- **Allowed paths:** `apps/litecoworkd/src/agents/module_registry.rs`, `apps/litecoworkd/src/agents/mod.rs`, `apps/litecoworkd/src/agents/modules/opencode.rs` (new), `apps/litecoworkd/tests/fixtures/agent-modules/opencode-2.0.32.json` (new), `apps/litecoworkd/src/operator.rs` (registry construction in state setup only).
- **Contract:** SPEC §5 B-11/B-12, §7.1, §13, AC-012, AC-017; docs/AGENT-CONTROL.md.
- **Change intent:** Register one trusted manifest: module `opencode`, version pinned to CLI 2.0.32, `supported_os` Windows and Linux, `supported_architectures` X86_64, descriptor API version, lifecycle features all `false` (no install/update in V1), TTL and probe budget from B-12, credential slots empty (native pass-through auth). Remove the file-level `dead_code` allow once routes call it. Registry selection uses exact identity (`registry_selects_only_an_exactly_compatible_module` behavior) and never a brand match.
- **Acceptance:**
  - [ ] Registry returns the OpenCode module only for 2.0.32 on Windows x86_64 or Linux x86_64; other OS/arch/versions return `ModuleSupportAssessment` unsupported.
  - [ ] Negative: a duplicate claim for the same identity fails closed (existing `duplicate_claims_for_same_compatible_identity_fail_closed`).
  - [ ] AC-012: the fixture pins the per-module probe budget; a fixture missing TTL/budget fails validation.
- **Command:** `cargo test -p litecoworkd --locked module_registry`
- **Evidence:** module_registry tests; fixture file path recorded in docs/IMPLEMENTATION-STATUS.md.
- **Stop:** stop if the manifest would need to claim a capability the supervisor does not exercise.

## 002-T04 - Runtime-local descriptor store with digest, TTL and probe budget

- **Status:** CODE-COMPLETE with one deviation 2026-11-10 (`AgentDescriptorStore` in storage-core, implemented over the existing `agent_control_descriptors` table: append with compare-and-swap on the prior digest, latest/current reads, TTL boundary at expiry, oversize and non-object bodies refused; tests `descriptor_*` in agent_binding_tests). Deviation: the 3-per-minute probe budget is an in-process sliding window in `agents/descriptor_service.rs`, not a stored ledger, so a daemon restart resets it.
- **Depends on:** T02, T03
- **Cross-package prerequisites:** NONE
- **Owner:** Storage, Agent Runtime.
- **Allowed paths:** `crates/storage-core/src/lib.rs` (AgentCatalogStore trait additions), `crates/storage-sqlite/src/agent_control_descriptors.rs` (new), `crates/storage-sqlite/src/lib.rs` (module registration and migration hook), `crates/storage-sqlite/tests/agent_control_descriptors.rs` (new), `docs/schemas/sqlite-vNN.sql` (from T02).
- **Contract:** SPEC §5 B-12, §7.2 `AgentControlDescriptorFreshness`, §11, §12, DATA-MODEL "Lifecycle and consistency", AC-008, AC-012.
- **Change intent:** Add `put_agent_control_descriptor` (insert-or-replace by `(agent_profile_id, module_version)` with CAS on prior `descriptor_digest`), `current_agent_control_descriptor(profile, now)` returning `None` past `expires_at`, and `admit_descriptor_probe(module_id, now)` enforcing at most 3 per minute. Store only the validated non-secret descriptor JSON, digest and times. No Workspace id. No network/process call inside the transaction.
- **Acceptance:**
  - [ ] AC-008: a descriptor at `issued_at + 900 s - 1 s` is fresh; at `+ 900 s` it is stale and not returned as current.
  - [ ] AC-012: the fourth probe admission inside one minute is refused without blocking other modules.
  - [ ] Concurrency: two writers with the same prior digest; exactly one commits, the other gets a version conflict.
  - [ ] Negative: a descriptor that fails `control_descriptor.rs` validation (oversize over 256 KiB, unknown field) is never stored.
  - [ ] Recovery: a crash between probe and put leaves the prior descriptor current and no partial row (transaction test).
- **Command:** `cargo test -p storage-sqlite --locked agent_control_descriptor` (filter to be added by this task)
- **Evidence:** storage tests; migration file name in CURRENT-RUN.md.
- **Stop:** stop if the store would need secret material or a Workspace-scoped copy.

## 002-T05 - AgentControlDescriptor producer and served descriptor routes

- **Status:** PARTIAL (descriptor routes `getAgentControlDescriptor` and `refreshAgentProfileControl` now served for a profile whose provider selects the OpenCode module: bounded probe outside any transaction, budget `429 RATE_LIMITED`, `Cache-Control: no-store`, Codex/unknown profiles still `503`; route tests added. Not done: `getAgentSessionOptions` and `getAgentHarnessCapabilities` projections (still target-only; serving them changes the 176/1/55 route counts), `StoredAgentDescriptorSource` for the credential route, panel stale/refresh test, and a live-OpenCode refresh).
- **Depends on:** T01, T04
- **Cross-package prerequisites:** NONE
- **Owner:** Agent Runtime, Desktop UI.
- **Allowed paths:** `apps/litecoworkd/src/agent_control_operator.rs`, `apps/litecoworkd/src/agent_credentials_routes.rs` (descriptor source only), `apps/litecoworkd/src/agents/control_descriptor.rs`, `apps/litecoworkd/src/agents/modules/opencode.rs`, `apps/litecoworkd/src/agents/opencode_profile_probe.rs`, `apps/litecowork-ui/src-tauri/src/agent_control_bridge.rs`, `apps/litecowork-ui/src/agents/AgentControlDescriptorPanel.tsx`, `apps/litecowork-ui/src/agents/desktop-agent-control-api.ts`, `apps/litecowork-ui/tests/agent-control-descriptor-panel.test.mjs`.
- **Contract:** SPEC §6 actor sequence step 4, §7.2 `StoredAgentDescriptorSource`, AC-014, AC-008, AC-012; docs/AGENT-CONTROL.md §12; OpenAPI `getAgentControlDescriptor`, `refreshAgentProfileControl`, `getAgentSessionOptions`, `getAgentHarnessCapabilities`.
- **Change intent:** Build the OpenCode descriptor from the module manifest plus a fresh probe: `installation` from the inventory observation, `auth` from the T07 observation (until T07 lands: `UNKNOWN`), `session_options` from the probe's `configured_providers` model list as module-declared options (provider id + model id, no cross-provider mixing), `input_surface` with text only and an empty native command list unless OpenCode 2.0.32 exposes a command list the module can read and the supervisor can execute, `capabilities` matching `opencode_endpoint_capabilities()`. Validate with the existing validator, compute digest, store via T04. `refresh_control_descriptor` runs the bounded probe outside any transaction under the T04 budget; `get_control_descriptor` returns the current stored descriptor or `503` when none is fresh. Replace `NoAgentDescriptorSource` with `StoredAgentDescriptorSource`. Serve `getAgentSessionOptions` and `getAgentHarnessCapabilities` as projections of the same descriptor. The daemon owns freshness derivation; the UI only renders it.
- **Acceptance:**
  - [ ] AC-014: a profile with the OpenCode module returns `200` with a validated descriptor and digest; a Codex or unknown profile still returns `503 DEPENDENCY_UNAVAILABLE` (existing test `profile_catalog_presence_never_manufactures_a_control_descriptor` keeps passing for module-less profiles).
  - [ ] AC-008: the panel shows "stale" and offers Refresh after TTL; refresh preserves any composer draft.
  - [ ] AC-012: a refresh over budget returns a typed throttle error; other profile cards stay interactive.
  - [ ] Security: the response carries `Cache-Control: no-store`; no executable path, native handle or secret appears in the payload or logs.
  - [ ] Concurrency: refresh racing a read never returns a half-written descriptor.
- **Command:** `cargo test -p litecoworkd --locked agent_control_operator` ; `cargo test -p litecoworkd --locked agent_credentials` ; `cargo test -p litecowork-ui --locked agent_control_bridge` ; `pnpm --dir apps/litecowork-ui test`
- **Evidence:** route tests, panel test; served-route inventory diff (descriptor routes move from fail-closed to served for one module).
- **Stop:** stop if any descriptor field would be inferred from a brand, catalog entry or RuntimeOffer rather than the module.

## 002-T06 - Fresh-descriptor gate on binding enable and admission

- **Status:** PARTIAL (enable gate enforced inside the enable transaction when the profile's module declares a descriptor: stale, missing or superseded revision fails with `DESCRIPTOR_STALE` mapped to `409 RESOURCE_STALE` and no write; exact replay returns the receipt; desktop enable reads or refreshes the descriptor and sends the revision. Not done: stale-option rejection at Send admission (AC-005), Enable-failure reason and Refresh focus return in the UI, `src-tauri` build check because gdk-3.0 is missing here).
- **Depends on:** T05
- **Cross-package prerequisites:** Feature 001 composer admission and draft preservation (Wave 1) for the AC-005 send-side half.
- **Owner:** Storage, Agent Runtime, Conversation Runtime.
- **Allowed paths:** `crates/storage-core/src/lib.rs` (`AgentBindingEnableRequest`), `crates/storage-sqlite/src/lib.rs` (`enable_agent_binding_transaction`, `validate_agent_binding_enable`), `crates/storage-sqlite/src/agent_binding_tests.rs`, `apps/litecoworkd/src/operator.rs` (`enable_agent_binding` handler), `apps/litecowork-ui/src-tauri/src/lib.rs` (`enable_agent_binding` command), `apps/litecowork-ui/src/agents/AgentCatalogSettings.tsx`, `apps/litecowork-ui/tests/agent-binding-mutation.test.ts`.
- **Contract:** SPEC §7.2 `EnableBindingWithDescriptor`, §11, AC-004, AC-005, AC-008.
- **Change intent:** Enable requires `descriptor_revision` equal to the current fresh descriptor digest for the bound profile's module version, checked inside the same transaction as the binding CAS; stale or missing returns `DescriptorStale` with no state change. Selected binding-scoped model/effort defaults are validated against the descriptor's declared options; a removed option rejects new admission with the existing stale-option error and leaves the stored choice untouched.
- **Acceptance:**
  - [ ] AC-004: enable with a fresh matching descriptor succeeds and writes one audit record; exact replay returns the original receipt and no second audit.
  - [ ] AC-004 negative: stale, mismatched-version or missing descriptor fails with no binding/event/audit write.
  - [ ] AC-005: an option removed from the fresh descriptor rejects admission; the composer draft is preserved (with Feature 001 composer test).
  - [ ] Race: descriptor refresh committed between UI read and enable makes enable fail with `DescriptorStale`, never enable against the older revision.
  - [ ] UI: Enable shows the exact reason and a Refresh descriptor action; focus returns to the Enable control after refresh.
- **Command:** `cargo test -p storage-sqlite --locked agent_binding_tests` ; `cargo test -p litecoworkd --locked enable_agent_binding` ; `pnpm --dir apps/litecowork-ui test`
- **Evidence:** storage and route tests; docs/IMPLEMENTATION-STATUS.md AgentBinding row updated to "enable gate enforced".
- **Stop:** stop if the gate cannot be evaluated inside the binding transaction without a network or process call.

## 002-T07 - Native sign-in begin and auth observation

- **Status:** PARTIAL 2026-11-10 (CODE + SERVED, fail-closed). `beginAgentAuthentication` is served by `agent_auth_operator.rs` and `AgentAuthService` (`agents/auth_service.rs`): it acts only on a method declared by the profile's fresh module descriptor, returns the in-flight challenge instead of a second launch, records `AUTHENTICATING`/`NEEDS_AUTH`/`ERROR` observations, reads an unreported or expired observation back as `UNKNOWN` (never success), refuses an undeclared method with `INVALID_ARGUMENT`, and reseals the observed state into the next descriptor. Observations are Runtime-local and in-memory (no migration); a restart returns `UNKNOWN`. Deviations and open items: the production `NativeLoginLauncher` is `UnqualifiedLauncher`, so every declared method is refused (503) and the OpenCode module still declares no auth method, because a native login cannot be launched or verified without a live OpenCode 2.0.32 binary; no completion re-probe hook is wired to a module; no Tauri command exists for Sign in (`src-tauri` cannot be built here, gdk-3.0 missing); the SPEC AuthObservation DDL is not added. UI: `DescriptorSummary` shows Sign in only for a declared `NATIVE_FLOW` method and an Authenticating text state. Evidence: `agent_auth_*` and `agent_auth_begin_*` tests (litecoworkd), `agent-control-descriptor-panel.test.mjs`. Windows launch DEFERRED (Windows rows are deferred to laptop testing).
- **Depends on:** T05
- **Cross-package prerequisites:** Feature 011 authenticated IPC and OS-open helper (Wave 0).
- **Owner:** Agent Runtime, Security/Trust, Desktop UI.
- **Allowed paths:** `apps/litecoworkd/src/agent_auth_operator.rs` (new), `apps/litecoworkd/src/operator.rs` (route merge only), `apps/litecoworkd/src/agents/modules/opencode.rs`, `crates/storage-core/src/lib.rs`, `crates/storage-sqlite/src/agent_auth_observations.rs` (new), `docs/schemas/sqlite-vNN.sql` (from T02), `apps/litecowork-ui/src-tauri/src/agent_control_bridge.rs`, `apps/litecowork-ui/src/agents/AgentCatalogSettings.tsx`, `apps/litecowork-ui/tests/agent-catalog-settings.test.mjs`.
- **Contract:** SPEC §6 step 3, §7.2 `AgentAuthObservation`, §10, AC-003, AC-016; docs/AGENT-ACCOUNT-ROUTE-MATRIX.md (only QUALIFIED routes offered as ready).
- **Change intent:** Serve `beginAgentAuthentication` for module-declared auth methods only. For OpenCode the module declares its own native login flow; the daemon launches it as the native harness process for the user to complete (never capturing credentials), then records an `AgentAuthObservation` after a bounded re-probe. Auth state is derived only from module-reported signals; the provider catalog's connected-provider list is labelled as reported, not as authentication proof, unless the module conformance fixture says otherwise. Add a read of the latest observation and feed it into the T05 descriptor `auth` block.
- **Acceptance:**
  - [ ] AC-016: begin with a declared method returns `AUTHENTICATING`; completion re-probes and records one of the closed states with `observed_at`/`expires_at`.
  - [ ] AC-003: cancel or failure records `ERROR`/`NEEDS_AUTH`, persists no secret bytes and leaves every binding unchanged (row-count assertion).
  - [ ] Negative: an undeclared method id returns `MethodNotDeclared`; a request with a caller path or shell string is rejected.
  - [ ] Concurrency: a second begin while one is `AUTHENTICATING` returns the in-flight observation, not a second native launch.
  - [ ] UI: Sign in is shown only when declared; Authenticating/Expired/Denied are distinct text states reachable by keyboard and announced to screen readers.
- **Command:** `cargo test -p litecoworkd --locked agent_auth` (filter to be added by this task) ; `cargo test -p storage-sqlite --locked agent_auth_observation` (filter to be added by this task) ; `pnpm --dir apps/litecowork-ui test`
- **Evidence:** tests; served-route inventory shows `beginAgentAuthentication` served.
- **Stop:** stop if the native login cannot be launched without LiteCowork handling the credential, or if auth state would have to be guessed.

## 002-T08 - Replay-safe credential slot submission

- **Status:** BLOCKED (retryable) 2026-11-10: the durable idempotency receipt needs a new additive table (a new migration; v31 and v32 are now taken by 014-T03 and 012-T05, so this is v33 per the allocation table in docs/LITECOWORK-BUILD-TASKS.md) and the matching `SecretStore` crash-reconcile path. Adding a migration touches the version-gated migration matrix, the failpoint suite and `docs/STORAGE.md`, and needs the full storage suite (about 10 minutes) plus the OS credential-store qualification script on the release-matrix OS, so it was not shipped in this slice. The route keeps returning `503 DEPENDENCY_UNAVAILABLE` rather than ship a non-idempotent secret write (this task's stop rule). OpenCode and Claude Code declare no `LITECOWORK_SECRET_STORE` slot in V1, so no user flow is affected. `StoredAgentDescriptorSource` is also still not wired to the credential route.
- **Depends on:** T05
- **Cross-package prerequisites:** Feature 011 SecretStore qualification on the release-matrix OS (`scripts/qualify-os-credential-store.sh`).
- **Owner:** Security/Trust, Storage.
- **Allowed paths:** `apps/litecoworkd/src/agent_credentials_operator.rs`, `apps/litecoworkd/src/agent_credentials_routes.rs`, `crates/storage-sqlite/src/trust.rs` (idempotency receipt only), `crates/storage-sqlite/src/lib.rs`.
- **Contract:** SPEC §7.2 `CredentialBroker`, AC-009, AC-010.
- **Change intent:** Persist a credential idempotency receipt (key, request digest without secret bytes, resulting SecretRef) before returning; exact replay returns the stored SecretRef without a second write; a changed body with the same key returns `ReplayConflict`. Enabled only for descriptor slots owned by `LITECOWORK_SECRET_STORE`. OpenCode and Claude Code declare no such slot in V1, so the route stays reachable but returns the typed unsupported error for them.
- **Acceptance:**
  - [ ] AC-009: bytes go only to SecretStore; response, logs and events contain slot id and SecretRef only; a `NATIVE_AGENT` slot never yields a SecretRef.
  - [ ] AC-010: exact replay returns the same SecretRef; changed body conflicts; key outside 1..=200 printable ASCII is rejected.
  - [ ] Crash: a crash after the SecretStore write and before the receipt is reconciled on retry as the same SecretRef, not a duplicate secret.
- **Command:** `cargo test -p litecoworkd --locked agent_credentials` ; `bash scripts/qualify-os-credential-store.sh`
- **Evidence:** tests and credential-store qualification log per OS.
- **Stop:** stop if exact replay cannot be proven; keep `503` rather than ship a non-idempotent secret write.

## 002-T09 - Composer model, effort and declared native commands from the descriptor

- **Status:** PARTIAL 2026-11-10 (CODE, UI view-model only). `conversation-model-picker.ts` adds `deriveModelPicker`, `buildModelSelection` and `deriveCommandList`: descriptor-backed model list with selected check and provider-default marker, Effort row only when declared, distinct LOADING/ERROR/DENIED/OFFLINE/STALE/BLOCKED/EMPTY texts, listbox announcement text, a payload `{optionId, providerModelId, descriptorRevision}` that refuses an undeclared model, and the "No commands declared by this agent" slash state. The placeholder "/ for actions" hint was already removed (guarded by `conversation-empty-state-truthfulness.test.mjs`). Not done, per this task's stop rule (no picker without a consuming send path): the popover component is not mounted in `ConversationsPage.tsx`, `getAgentSessionOptions` is not served, and the dispatcher does not revalidate the option at admission. Tests: `tests/conversation-model-picker.test.ts` (6).
- **Depends on:** T05, T06
- **Cross-package prerequisites:** Feature 001 composer and Send admission (Wave 1); Feature 004 consumes the same picker gating.
- **Owner:** Desktop UI, Agent Runtime.
- **Allowed paths:** `apps/litecowork-ui/src/conversations/ConversationsPage.tsx` (composer picker region only), `apps/litecowork-ui/src/conversations/conversation-agent-surface.ts`, `apps/litecowork-ui/src/conversations/conversation-agent-selection.ts`, `apps/litecowork-ui/src/agents/desktop-agent-control-api.ts`, `apps/litecowork-ui/src-tauri/src/agent_control_bridge.rs`, `apps/litecowork-ui/tests/conversation-agent-surface.test.ts`, `apps/litecowork-ui/tests/conversation-agent-selection.test.ts`.
- **Contract:** SPEC "V1 model and effort picker details", AC-005, AC-015, AC-008; contracts/ui-behavior.md; docs/NATIVE-COMPOSER-LLD.md.
- **Change intent:** Read `getAgentSessionOptions` for the selected binding and render the compact anchored model popover (search, selected check, provider default marker, Effort row only when declared). Submit `{optionId, providerModelId, descriptorRevision}` with the turn. Render a native command list only from descriptor `input_surface` commands; when none are declared, the "/" affordance shows "No commands declared by this agent" and inserts nothing. Remove the placeholder hint text that implies commands exist. TypeScript only parses and renders the daemon projection; it does not recompute freshness.
- **Acceptance:**
  - [ ] AC-015: options and commands appear only when declared; undeclared or stale affordances are disabled with a reason.
  - [ ] AC-005: a stale selection is rejected at admission, the draft and previous committed choice are kept and a Refresh action is offered.
  - [ ] Loading, empty (no options), error, denied, offline and stale states each render distinct text.
  - [ ] Accessibility: the popover is reachable by keyboard, traps and restores focus, uses listbox semantics and announces the selected model.
  - [ ] Negative: options from another provider/account never mix; switching model never mutates a running turn.
- **Command:** `pnpm --dir apps/litecowork-ui test` ; `pnpm --dir apps/litecowork-ui run build`
- **Evidence:** UI tests; docs/V1-UI-ACTION-STATE-LEDGER.md rows for the picker updated by T16.
- **Stop:** stop if the selected option cannot be revalidated at admission by the dispatcher (no picker without a consuming send path).

## 002-T10 - Supply 002 operations to the Feature 011 Settings -> Agents screen

- **Status:** PARTIAL 2026-11-10 (CODE + SERVED, fail-closed). `openAgentNativeConfiguration` is served by `agent_native_config_operator.rs`: the caller supplies only a `targetId`; an id with a path separator or control character is rejected, a target not declared in the fresh module descriptor is rejected before any opener is consulted, and the production `UnqualifiedOpener` refuses declared targets with 503. UI: `DescriptorSummary` shows five independent axes (Detection, Installation, Configuration, Authentication, Readiness), declared Sign in and Open native config controls only, a disabled state when no desktop command is supplied, and the "native harness state" label for MCP/hooks/extensions (AC-007, AC-011). Not done: Tauri commands in `agent_binding_bridge.rs` (`src-tauri` cannot be built here), per-card probe loading test (AC-012 UI), and a real opener. Tests: `native_config_open_rejects_undeclared_targets_and_caller_paths`, `agent-control-descriptor-panel.test.mjs`.
- **Depends on:** T05, T07
- **Cross-package prerequisites:** Feature 011 Settings -> Agents screen and control table (A-35 owner). This task links to that screen and does not republish it.
- **Owner:** Desktop UI (operation wiring), Feature 011 (screen).
- **Allowed paths:** `apps/litecowork-ui/src/agents/AgentCatalogSettings.tsx`, `apps/litecowork-ui/src/agents/AgentControlDescriptorPanel.tsx`, `apps/litecowork-ui/src-tauri/src/agent_binding_bridge.rs` (new; new 002 commands go here, not into `lib.rs`), `apps/litecowork-ui/src-tauri/src/lib.rs` (command registration only), `apps/litecoworkd/src/agent_native_config_operator.rs` (new, `openAgentNativeConfiguration`), `apps/litecowork-ui/tests/agent-catalog-settings.test.mjs`.
- **Contract:** SPEC "V1 UI and lifecycle contract", A-35 note in §5, AC-007, AC-011, AC-012; [Feature 011 SPEC](../011-desktop-shell-ipc-settings-runtime/SPEC.md); contracts/ui-behavior.md.
- **Change intent:** Expose to the 011 screen: independent Detection/Installation/Configuration/Authentication/Readiness chips from real observations; the descriptor-driven Capabilities matrix; Sign in (T07); Open native config for module-known target ids only; native MCP/hooks shown as "native harness state" with no Core Effect claim. Per-card loading so one slow probe does not block the page.
- **Acceptance:**
  - [ ] AC-011: the five axes render independently with keyboard and screen-reader access; no fake Install, Sign in, Test or Ready.
  - [ ] AC-007: native-only MCP/hooks are labelled native harness state.
  - [ ] AC-012: a probe held for its full timeout leaves other cards interactive.
  - [ ] Negative: Open native config rejects a caller-supplied path; disabled when the descriptor declares no target.
  - [ ] States: loading, empty, error, denied, offline, stale and recovery are distinct.
- **Command:** `pnpm --dir apps/litecowork-ui test` ; `cargo test -p litecoworkd --locked native_config` (filter to be added by this task)
- **Evidence:** UI tests and the Feature 011 screen test that consumes these operations.
- **Stop:** stop if a control would be visible without an owning served route.

## 002-T11 - Catalog, inventory and honest install/update unavailability for V1

- **Status:** PARTIAL 2026-11-10 (CODE). Registry and inventory remain served; Install and Update are shown as unavailable with the reason "No qualified installer for this module in V1" and no Install or Update control exists to focus (test `install and update are visibly unavailable and no install control is offered`). Not done: the "Install guide" external link (no verified guide URL per module), and "Add existing installation" through the native picker plus exact module validation.
- **Depends on:** T03
- **Cross-package prerequisites:** NONE
- **Owner:** Agent Runtime, Desktop UI.
- **Allowed paths:** `apps/litecoworkd/src/agent_registry_operator.rs`, `apps/litecoworkd/src/agents/native_cli.rs`, `apps/litecoworkd/src/operator.rs` (`list_agent_installations` only), `apps/litecowork-ui/src/agents/AgentCatalogSettings.tsx`, `apps/litecowork-ui/tests/agent-candidate-catalog.test.mjs`.
- **Contract:** SPEC §3, §17, AC-001, AC-002, AC-012; docs/V1-AGENT-DISCOVERY-AND-CATALOG-LLD.md.
- **Change intent:** Keep registry results as discoverable distributions; show Install/Update as visibly unavailable with the reason "No qualified installer for this module in V1" and an "Install guide" external link. Add "Add existing installation" only through the native picker plus exact module validation. No registry-provided shell string is executed.
- **Acceptance:**
  - [ ] AC-001: registry rows are never labelled installed, authenticated or ready.
  - [ ] AC-002: an inventory hit creates no binding or profile.
  - [ ] AC-012: inventory probes are bounded per candidate.
  - [ ] Negative: no Install control is focusable as an enabled action.
- **Command:** `cargo test -p litecoworkd --locked agent_registry` ; `pnpm --dir apps/litecowork-ui test`
- **Evidence:** tests; Settings ledger row "Install/Update: unavailable in V1".
- **Stop:** stop if any install path would run outside a module-owned, digest-verified plan.

## 002-T12 - Claude Code CLI as the second AgentModule (B-11)

- **Status:** BLOCKED (retryable) 2026-11-10: the module needs the exact Claude Code CLI version, its non-interactive streaming transport and native auth method pinned from an installed CLI, and none is installed in this sandbox. Shipping a manifest with a guessed version pin would violate AC-017 and the T12 stop rule, so `CLAUDE_CODE` stays inventory-only, `NativeProviderKey::ClaudeCode` still selects no module and the probe allowlist is unchanged. Windows x64 pinning is DEFERRED to laptop testing; Linux x64 pinning needs the CLI installed.
- **Depends on:** T03, T05, T07
- **Cross-package prerequisites:** Feature 004 Claude Code dispatch arm in `AgentSessionSupervisor` (Wave 4) consumes this module; Feature 001 supervisor durability.
- **Owner:** Agent Runtime, Security/Trust.
- **Allowed paths:** `apps/litecoworkd/src/agents/modules/claude_code.rs` (new), `apps/litecoworkd/src/agents/claude_code_profile_probe.rs` (new), `apps/litecoworkd/src/agents/native_provider.rs`, `apps/litecoworkd/src/agents/mod.rs`, `apps/litecoworkd/src/operator.rs` (`probe_local_agent_profile` provider allowlist only), `apps/litecoworkd/tests/fixtures/agent-modules/claude-code.json` (new), `docs/schemas/operator-api.openapi.yaml` (probe enum), `apps/litecowork-ui/src/agents/AgentCatalogSettings.tsx`.
- **Contract:** SPEC §5 B-11, AC-017, AC-013 (same offer rules), AC-014, AC-016; docs/AGENT-ACCOUNT-ROUTE-MATRIX.md.
- **Change intent:** Add `CLAUDE_CODE` to the profile probe allowlist (`"CODEX" | "OPENCODE"` today); a bounded read-only probe that reads version and module-declared auth status without inference; a conformance fixture pinning the exact CLI version, non-interactive streaming transport and native auth method observed from the installed CLI on each OS; descriptor production through T05; sign-in through T07. Offer compatibility follows the same computed predicate as T01, keyed on the pinned version.
- **Acceptance:**
  - [ ] AC-017: Claude Code has its own installation, auth, descriptor, binding and readiness evidence; no field is copied from OpenCode.
  - [ ] Negative: an unpinned Claude Code version stays `compatible=false`; Codex remains probe-only and not dispatchable.
  - [ ] AC-016: Claude Code sign-in uses its native flow; no credential is captured.
- **Command:** `cargo test -p litecoworkd --locked claude_code` (filter to be added by this task) ; `pnpm --dir apps/litecowork-ui test`
- **Evidence:** fixture file, probe transcript per OS in CURRENT-RUN.md.
- **Stop:** stop if the CLI requires credential copying, cookie reuse or a hidden proxy (fail closed per AGENT-ACCOUNT-ROUTE-MATRIX).

## 002-T13 - Failure, restart and race recovery for probe, descriptor, auth and binding

- **Status:** PARTIAL 2026-11-10 (CODE). Added `agent_lifecycle_recovery_restart_revalidates_the_descriptor_until_reprobed` (a new RuntimeIncarnation never inherits a descriptor until re-probed) and `agent_lifecycle_recovery_failed_probe_leaves_the_prior_descriptor_intact` (crash or unpinned probe leaves the prior descriptor current and no partial row) in `descriptor_service.rs`; auth unknown and cancel are covered by `agent_auth_*`; descriptor race and TTL by existing `descriptor_*` storage tests. The planned integration file `tests/agent_lifecycle_recovery.rs` was not created because `litecoworkd` is a binary crate whose internals are not importable from `tests/`. Not done: credential replay after crash (blocked with T08), duplicate create/enable replay beyond existing receipt tests, and the `scripts/v1_resilience_matrix.py` row.
- **Depends on:** T05-T08
- **Cross-package prerequisites:** Feature 011 Runtime incarnation and restart observation.
- **Owner:** Agent Runtime, Storage.
- **Allowed paths:** test files only: `apps/litecoworkd/tests/agent_lifecycle_recovery.rs` (new), `crates/storage-sqlite/src/agent_binding_tests.rs`, `crates/storage-sqlite/tests/agent_control_descriptors.rs`; plus fixes inside the files owned by T01, T04-T08.
- **Contract:** SPEC §11, §12, WIRING.yaml `recovery`, AC-006, AC-003, AC-010.
- **Change intent:** Exercise: daemon restart (new RuntimeIncarnation invalidates offers and descriptors until re-probed); crash mid-probe (prior binding and descriptor intact, failed observation recorded); probe timeout; duplicate create/enable replay; cancel of sign-in racing completion; descriptor refresh racing enable; offer expiry while a turn is being admitted.
- **Acceptance:**
  - [ ] AC-006: after restart install, auth and descriptor freshness are revalidated independently before a binding shows usable.
  - [ ] AC-003: a cancelled sign-in during restart leaves bindings unchanged.
  - [ ] AC-010: credential replay after crash returns the original SecretRef.
  - [ ] An UNKNOWN probe or auth outcome is shown as unknown, never as success.
- **Command:** `cargo test -p litecoworkd --locked agent_lifecycle_recovery` (filter to be added by this task) ; `cargo test -p storage-sqlite --locked agent_binding_tests`
- **Evidence:** recovery test report in CURRENT-RUN.md; `scripts/v1_resilience_matrix.py` row for agent lifecycle.
- **Stop:** stop if any recovery path would infer success from an intent record.

## 002-T14 - Cross-layer trace for every 002 user-visible operation

- **Status:** PARTIAL 2026-11-10 (DOCS). Trace table for Probe, Create/Enable/Disable binding, Refresh descriptor, Sign in, Select model and Open native config is in `contracts/ui-behavior.md` with real symbol names and test ids. Three breaks are recorded rather than hidden (no Tauri command for Sign in, Open native config or the picker; `getAgentSessionOptions` still target-only; no qualified launcher or opener). `docs/V1-UI-ACTION-STATE-LEDGER.md` rows were not added because that ledger is keyed by UIV1 ids owned by the documentation owner.
- **Depends on:** T09, T10, T13
- **Cross-package prerequisites:** NONE
- **Owner:** Agent Runtime, Desktop UI.
- **Allowed paths:** this package (`contracts/ui-behavior.md`, `WIRING.yaml`), `docs/V1-UI-ACTION-STATE-LEDGER.md` (rows for 002 controls).
- **Contract:** AGENTS.md §4; AC-011, AC-013, AC-014, AC-015, AC-016.
- **Change intent:** For Probe profile, Create binding, Enable/Disable, Refresh descriptor, Sign in, Select model/effort and Open native config, record: control -> Tauri command -> Operator route -> authorization -> service -> transaction/CAS -> record/event/receipt -> projection -> restart behavior -> evidence test. Any break found becomes a fix in the owning task before qualification.
- **Acceptance:**
  - [ ] Every listed control has a complete trace with real symbol names and a test id.
  - [ ] No UI control lacks an owning route; no served route lacks a UI path or an explicit "no UI by design" note.
- **Command:** `python3 .specify/scripts/validate_feature_package.py` ; `python3 scripts/audit_served_routes.py`
- **Evidence:** trace tables in contracts/ui-behavior.md.
- **Stop:** stop if a trace step cannot be named from source.

## 002-T15 - Release-matrix qualification (CODE, SYSTEM, USER)

- **Status:** DEFERRED 2026-11-10 by owner instruction: release-matrix qualification (Windows x64 and Linux x64 packaged runs, owner USER evidence) depends on Windows hardware, the packaged installer and release items, and is to be tested later on the owner's laptop. No QUALIFIED claim is made anywhere in this package.
- **Depends on:** T12, T14
- **Cross-package prerequisites:** Feature 019 release matrix rows and packaged installer; Feature 001 E2E-01 Send path; Feature 004 E2E-02 relay for the Claude Code row.
- **Owner:** Agent Runtime, Product (owner USER evidence).
- **Allowed paths:** evidence only: `CURRENT-RUN.md`, `docs/IMPLEMENTATION-STATUS.md` (002 rows), evidence bundle paths defined by docs/ACCEPTANCE-AND-E2E-PLAN.md.
- **Contract:** docs/ACCEPTANCE-AND-E2E-PLAN.md E2E-01 (detect, sign in, descriptor, binding portion) and E2E-02 (second module); AC-001 to AC-017.
- **Change intent:** On the packaged desktop, Windows x64 with OpenCode 2.0.32: detect, probe, sign in, refresh descriptor, create and enable binding, choose model, Send through Feature 001, restart and revalidate. Repeat on Linux x64. Then Claude Code CLI on Windows x64 and Linux x64. Include denial (stale descriptor, cancelled sign-in, incompatible version), duplicate replay and restart. WSL2 is not evidence for either row.
- **Acceptance:**
  - [ ] CODE: all task commands green at one recorded revision.
  - [ ] SYSTEM: packaged desktop run per row with daemon, IPC, storage and real harness.
  - [ ] USER: owner performs the E2E-01 binding portion and records pass/fail per row.
  - [ ] Denial and recovery cases recorded per row.
- **Command:** `python3 scripts/v1_platform_matrix.py` ; `cargo test -p litecoworkd --locked` ; `pnpm --dir apps/litecowork-ui test`
- **Evidence:** evidence bundle per row with build, OS, harness version and auth route.
- **Stop:** stop if any row passes only under WSL2, a mock harness or a non-packaged build.

## 002-T16 - Documentation and ledger sync

- **Status:** PARTIAL 2026-11-10 (DOCS). `served-route-inventory.json` regenerated (178 served / 1 local-only / 53 target-only; three audit scripts, `tests/test_architecture_validator.py`, IMPLEMENTATION-STATUS, the register, coverage and semantic maps updated), `docs/API.md` and `contracts/api-events.md` state the two newly served fail-closed routes. SPEC section 4 and the function ledger F13 rows were not rewritten because T15 produced no evidence; no claim is above CODE/SERVED. Closing register rows B-11/B-12 is left to the orchestrator.
- **Depends on:** T15
- **Cross-package prerequisites:** orchestrator-owned register and traceability updates.
- **Owner:** Agent Runtime, documentation owner.
- **Allowed paths:** `docs/IMPLEMENTATION-STATUS.md`, `docs/schemas/served-route-inventory.json` (regenerated), `docs/V1-SETTINGS-CONTRACT-LEDGER.md`, `docs/V1-UI-ACTION-STATE-LEDGER.md`, `docs/V1-FUNCTION-CONTRACT-LEDGER.md` (F13 rows), this package.
- **Contract:** AGENTS.md §1 item 6; SPEC §4.
- **Change intent:** Update SPEC §4 and IMPLEMENTATION-STATUS with the new evidence level per capability (never above what T15 recorded), regenerate the served-route inventory, update UI ledgers and function ledger rows, and ask the orchestrator to close B-11/B-12 register rows as implemented.
- **Acceptance:**
  - [ ] Served-route inventory matches source (no 002 route listed served that is not mounted).
  - [ ] Every evidence claim cites a T15 record.
- **Command:** `python3 scripts/audit_served_routes.py` ; `python3 .specify/scripts/validate_feature_package.py`
- **Evidence:** regenerated inventory diff; IMPLEMENTATION-STATUS diff.
- **Stop:** stop if any doc would claim a level higher than recorded evidence.

## 002-T17 - Define the native-agent adapter as an ACP client (P-11)

- **Status:** PARTIAL 2026-11-10 (CODE, commit bf9b8e0). `agents/acp_client.rs` implements a bounded-framing ACP client (initialize, session/new, session/prompt, session/update, session/cancel, request_permission, fs and terminal routing to the Client owner, foreign-session traffic ignored, one cancel, fail-closed faults) with tests `acp_client_adapter_*` (6). Open: the ACP SDK package version and protocol revision are still not pinned in RESEARCH.md (no network fetch of the published SDK was done), so that acceptance box is unmet; the module is not wired to any dispatch path.
- **Depends on:** T03
- **Cross-package prerequisites:** Feature 001 adapter-trait task (Feature 001 T04) is the Conversation-side integration point; this package's Adapter 1.0 registry task T03 is the local integration point.
- **Owner:** Agent Runtime.
- **Allowed paths:** `apps/litecoworkd/src/agents/acp/` (new), `apps/litecoworkd/src/agents/module_registry.rs`, `apps/litecoworkd/src/agents/modules/`, `specs/002-native-agent-install-auth-binding/RESEARCH.md`, adjacent tests.
- **Contract:** SPEC §5, §7.1; AC-013, AC-018; AGENT-CONTROL; register A-54.
- **Change intent:** define the native-agent adapter as an ACP client covering `initialize`, `session/new`, `session/prompt`, `session/update`, `session/cancel`, and the client-side `request_permission`, filesystem read/write and terminal `create`/`output`/`kill`/`release` surfaces, with a per-provider launch catalog. Keep the OpenCode v2 HTTP path (Feature 001 T04 adapter) as one non-ACP adapter until OpenCode's own ACP route qualifies. Pin the ACP SDK version in RESEARCH.md. ACP agents stay native subprocesses that own reasoning and auth; LiteCowork stays the Client that owns permission, files, terminals and evidence (no invariant is weakened).
- **Acceptance:**
  - [ ] The ACP client completes `initialize`/`session/new`/`session/prompt`/`session/update`/`session/cancel` against a fixture agent and routes `request_permission`, fs and terminal calls to the Client owner (AC-018).
  - [ ] The per-provider launch catalog selects a provider by exact identity, never a brand match; OpenCode v2 HTTP remains a distinct non-ACP adapter (AC-013).
  - [ ] The pinned ACP SDK version is recorded in RESEARCH.md with its source.
  - [ ] Negative: an uncorrelated or foreign session update cannot mutate another session's state.
- **Command:** `cargo test -p litecoworkd --locked acp_client_adapter` (filter to be added by this task)
- **Evidence:** fixture test output; pinned SDK version in RESEARCH.md recorded in Completion evidence.
- **Stop:** stop if an ACP surface would let a native tool action be recorded as a LiteCowork Effect.

## 002-T18 - Agent Matchmaker ranking for a job (P-12)

- **Status:** PARTIAL 2026-11-10 (CODE). `agents/matchmaker.rs` `rank_agents` ranks eligible bindings by capabilities declared in fresh descriptors, gives a plain-language reason per candidate, excludes disabled, not-lead-eligible, missing and stale-descriptor candidates with a reason and no rank, keeps caller order on ties and always reports `advisory: true` (tests `agent_matchmaker_*`, 3). Not done: the Operator route (adds an OpenAPI operation and route counts), the UI surface in `apps/litecowork-ui/src/`, and the job-text need extraction is a fixed keyword table, not a model.
- **Depends on:** T05, T17
- **Cross-package prerequisites:** NONE
- **Owner:** Agent Runtime, UI.
- **Allowed paths:** `apps/litecoworkd/src/agents/matchmaker.rs` (new), `apps/litecoworkd/src/operator.rs` (route wiring only), `apps/litecowork-ui/src/`, adjacent tests.
- **Contract:** SPEC §5, §7.1; AC-019; AGENT-CONTROL.
- **Change intent:** given a job description, rank the eligible bound agents by declared capability from their fresh descriptors, each with a stated plain-language reason. The ranking is advisory: the user still chooses the agent. No ranking is produced from a stale descriptor.
- **Acceptance:**
  - [ ] Ranking uses only fresh descriptor-declared capabilities and lists a reason per candidate (AC-019).
  - [ ] The user selection is never overridden by the ranking; the chosen agent is the bound one (AC-019).
  - [ ] Negative: an ineligible or disabled binding is excluded; a stale descriptor yields no rank.
- **Command:** `cargo test -p litecoworkd --locked agent_matchmaker` (filter to be added by this task)
- **Evidence:** test output in Completion evidence.
- **Stop:** stop if the ranking would auto-dispatch a job without the user choosing.

## 002-T19 - Package VERIFIED for V1

- **Status:** NOT STARTED
- **Depends on:** T01-T18
- **Cross-package prerequisites:** Feature 001 and Feature 004 qualification records for the shared E2E rows.
- **Owner:** Product (owner sign-off), Agent Runtime.
- **Allowed paths:** this package (status lines), `CURRENT-RUN.md`.
- **Contract:** AGENTS.md §3 lifecycle; docs/SPECIFICATION-STANDARD.md; AC-001 to AC-017.
- **Change intent:** Assemble the evidence bundle (per-task revision, command, result, OS, harness version, auth route, reviewer), move status ACCEPTED/LOCKED -> IMPLEMENTING (when the first task starts) -> VERIFIED, and record owner sign-off.
- **Acceptance:**
  - [ ] Every AC-001 to AC-017 maps to a passing recorded test and, where user-visible, a USER record.
  - [ ] Owner sign-off recorded with date and revision.
- **Command:** `python3 .specify/scripts/validate_feature_package.py`
- **Evidence:** evidence bundle index in CURRENT-RUN.md.
- **Stop:** stop if any release-matrix row lacks USER evidence; the package stays IMPLEMENTING.

## Dependency order

T01 and T02 start in parallel (T01 is the first code change). T02 -> T03 -> T04. T01 + T04 -> T05. T05 -> T06, T07, T08. T05 + T06 -> T09. T05 + T07 -> T10. T03 -> T11. T03 + T05 + T07 -> T12. T05-T08 -> T13. T09 + T10 + T13 -> T14. T12 + T14 -> T15 -> T16. T03 -> T17 (ACP client, P-11); T05 + T17 -> T18 (Matchmaker, P-12). T16 + T18 -> T19 (VERIFIED).

## Cross-package dependencies

Cross-package task numbers are being re-authored in parallel, so this table names the owning package and outcome instead of a task number.

| This task | Other package (outcome) | Reason |
|---|---|---|
| T01 | Feature 001 (first native OpenCode Conversation route) | 001 cannot reach Send until an OpenCode binding can be created; T01 unblocks it. |
| T02 | Feature 011 (IPC/route conventions), orchestrator (migration numbering) | Shared route and migration sequence. |
| T06 | Feature 001 (composer admission, draft preservation) | Stale-option rejection is enforced at Send admission. |
| T07, T08 | Feature 011 (authenticated IPC, SecretStore qualification) | Native launch and secret custody need the qualified shell and store. |
| T09 | Feature 001 (composer), Feature 004 (picker gating) | Picker lives in the 001 composer; 004 reuses the gating. |
| T10 | Feature 011 (Settings -> Agents screen, A-35) | 011 owns the screen; 002 supplies operations. |
| T12 | Feature 004 (Claude Code dispatch arm) | 004 adds the supervisor arm that uses this module. |
| T15 | Feature 019 (release matrix, installer), Feature 001 (E2E-01), Feature 004 (E2E-02) | Packaged qualification rows. |

## Completion evidence

For each task record in CURRENT-RUN.md: task id, git revision, exact command(s), result (pass/fail with test counts), OS and build (Windows 11/10 x86_64-pc-windows-msvc or Linux x86_64), harness and version (OpenCode 2.0.32 or the pinned Claude Code CLI version), auth route, reviewer, and date. A task is DONE only when its acceptance boxes are checked against that record. Evidence levels follow AGENTS.md §5; a passing unit test is CODE, not QUALIFIED.
