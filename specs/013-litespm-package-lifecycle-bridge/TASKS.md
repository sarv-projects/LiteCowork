# Tasks - LiteSPM Package Lifecycle Bridge

**Feature:** 013 | **Status:** IN_REVIEW - not owner-accepted; T01 blocks every task; tasks not started unless marked.
**Spec revision:** 0.1 | **Release target:** V1 (release matrix rows: Windows x64 + OpenCode v2.0.32 first; Linux x64 second)
**Owners:** domain owners named in SPEC.md. Blockers B-36 and B-37 are PROPOSED, pending owner.

## Current baseline

Verified against the checkout on branch `docs/audit-competitor-superset` on 2026-11-09.

- **Operator:** `apps/litecoworkd/src/discover_operator.rs` (merged in `apps/litecoworkd/src/operator.rs` via `discover_operator::routes()`) serves `GET /v1/discover/search`, `GET /v1/discover/items/{item_id}`, `POST /v1/capabilities/{capability_id}/connect-or-install` (always `503 DEPENDENCY_UNAVAILABLE`, reason `LITESPM_PACKAGE_SERVICE_NOT_QUALIFIED`, `requested_scope_granted: false`, no receipt) and `POST /v1/capabilities/{capability_id}/test` (always `health: "UNKNOWN"`). Defect: `test_capability_connection` sets `checked_at` to `new_id("cor")` and `search_discover` passes the same kind of correlation ID as `now`. Test `capability_commands_require_an_idempotency_key_and_stay_truthful` lives in `apps/litecoworkd/src/routine_run_operator_tests.rs`.
- **Storage:** `crates/storage-sqlite/src/discover.rs` (`SqliteDiscoverStore::search_catalog`, `describe_entry`, `PACKAGE_SERVICE_NOT_QUALIFIED`, `DiscoverProviderStatus` default `package_service_qualified: false`, `DISCOVER_MAX_RUNTIME_CAPABILITIES = 50`) projects `SkillRecord` rows and Runtime capability hosts only. `crates/storage-core/src/discover.rs` defines `DiscoverCatalogEntry` (with `validate()` refusing `authoritative = true`), `DiscoverInstallState`, `DiscoverEntrySource::LiteSpmPackage`. No package approval or receipt table exists; the migration chain ends at `docs/schemas/sqlite-v28.sql`.
- **Domain:** `crates/domain-invocations/src/capability_bridge.rs` `classify_agent_capability_route` with six unit tests; no production caller.
- **Tauri:** `apps/litecowork-ui/src-tauri/src/discover_bridge.rs` `discover_request` supports operations `search` and `item` only (registered in `apps/litecowork-ui/src-tauri/src/lib.rs`).
- **UI:** `apps/litecowork-ui/src/discover/DiscoverPage.tsx` (tabs Explore, "Installed / native", Managed, Connections; Managed and Connections render "Not available"), `discover-state.ts` (`connectActionOutcome()` disables every row action), `discover-api.ts`, `desktop-discover-api.ts`; test `apps/litecowork-ui/tests/discover-catalog-state.test.ts`.
- **Reusable:** `crates/operator-ipc` (`BudgetedBody`, `InFlightBodyBudget`, framed `read_response`), `crates/windows-local-security` (`current_logon_sid`, `connect_operator_pipe`).
- **LiteSPM client:** absent. Upstream pins disagree (`9f42b25d`, `111fd9b8`, `f77a3a3`); upstream lacks a published descriptor, approval-recording method and package `operation.get` per interop LLD Appendix A.

Gaps to V1: the whole bridge (client, profile, inventory, plan/approval/apply/receipt, reconcile, adopt/unmanage, rollback/remove/update/repair, UI) plus CODE/SYSTEM/USER qualification on both rows.

## 013-T01 - Owner acceptance of SPEC and blockers

- **Status:** BLOCKED - B-36 and B-37 are PROPOSED; pending owner acceptance (repository owner, sarv-projects).
- **Depends on:** NONE
- **Cross-package prerequisites:** NONE (orchestrator records B-36/B-37 in the register and review packet).
- **Owner:** Product (repository owner); Capability Fabric reviewer.
- **Allowed paths:** this package; `docs/OWNER-ACCEPTANCE-REVIEW-PACKET.md` and the register rows (orchestrator only).
- **Contract:** SPEC sections 2, 3, 5, 17; AC-019; SPECIFICATION-STANDARD section 7.
- **Change intent:** owner accepts or amends B-36 (pin, transport, peer auth, approval rule) and B-37 (skill on OpenCode v2.0.32 user scope, Windows row first) and answers O-04 (who installs the LiteSPM service). On acceptance set SPEC `**Status:** ACCEPTED/LOCKED`, revision 1.0, and WIRING `status: "ACCEPTED/LOCKED"`.
- **Acceptance:**
  - [ ] B-36 and B-37 recorded ACCEPTED with date and owner, or amended text recorded.
  - [ ] SPEC and WIRING statuses match and the validator passes.
  - [ ] Negative: until this closes, every other task stays BLOCKED and AC-019 behaviour (fail-closed connect-or-install) is unchanged.
- **Command:** `python3 .specify/scripts/validate_feature_package.py`
- **Evidence:** owner acceptance line in SPEC metadata; register rows B-36/B-37.
- **Stop:** owner rejects the bridge approach; package becomes SUPERSEDED or BLOCKED.

## 013-T02 - Re-pin LiteSPM and record the bridge contract baseline (lane steps 0-1)

- **Status:** BLOCKED (owner acceptance, T01; upstream publication of descriptor/approval/operation-get, LiteSPM owner).
- **Depends on:** T01
- **Cross-package prerequisites:** NONE; upstream LiteSPM parity requests A1-A6 (LiteSPM owner).
- **Owner:** Capability Fabric; LiteSPM owner for the published contract.
- **Allowed paths:** `specs/013-litespm-package-lifecycle-bridge/RESEARCH.md`, `docs/LITESPM-LITECOWORK-INTEROP-LLD.md`, `docs/LITESPM-INTEGRATION-PLAN.md`, `docs/REFERENCE-IMPLEMENTATIONS.md`.
- **Contract:** SPEC section 5 B-36, section 17 O-01, O-05, O-06; AC-001, AC-003; AGENTS.md section 6.
- **Change intent:** read (do not build or run) the chosen LiteSPM revision; record the SHA, license, the published descriptor fields, wire operation names for inventory, describe, plan, approval recording, apply, operation get, adopt, remove, update, reconcile, unmanage, error vocabulary and peer-auth mechanism per OS. Replace the three disagreeing pins with one. Mark each semantic operation in SPEC 7.2 as ADVERTISED or ABSENT at that pin.
- **Acceptance:**
  - [ ] One pin recorded in RESEARCH.md and both LiteSPM owner docs; old pins marked historical.
  - [ ] Every wire name cites an upstream schema or source file at the pin; none comes from CLI output.
  - [ ] Negative: any operation without a published schema is recorded ABSENT and its verb stays UNAVAILABLE.
- **Command:** `python3 .specify/scripts/validate_feature_package.py`
- **Evidence:** RESEARCH.md pin table; diff of the two LiteSPM owner docs.
- **Stop:** upstream publishes no descriptor or approval operation; record observe-only scope and continue only T03-T06 and T13 in read-only form.

## 013-T03 - Promote accepted interfaces into owner docs and schemas

- **Status:** BLOCKED (owner acceptance, T01).
- **Depends on:** T01, T02
- **Cross-package prerequisites:** Feature 006 review of the connect-or-install split (O-03) and of the grant-invalidation notification.
- **Owner:** API, Storage, Events, Capability Fabric owners.
- **Allowed paths:** `docs/API.md`, `docs/EVENTS.md`, `docs/STORAGE.md`, `docs/SCHEMAS.md`, `docs/CAPABILITY-FABRIC.md`, `docs/schemas/operator-api.openapi.yaml`, `docs/schemas/error-codes.schema.json`, `docs/schemas/domain-event.schema.json`, `docs/schemas/sqlite-v29.sql` (new; use the next free number at implementation time), this package.
- **Contract:** SPEC sections 7.2, 8, 12; AC-001, AC-007, AC-019.
- **Change intent:** promote the seven routes in SPEC 7.2, error codes `LITESPM_PROTOCOL_INCOMPATIBLE`, `LITESPM_OPERATION_NOT_ADVERTISED`, `PACKAGE_HOST_PROFILE_UNSUPPORTED`, `PACKAGE_PLAN_EXPIRED`, `PACKAGE_APPROVAL_CONSUMED`, `PACKAGE_OPERATION_UNKNOWN_PENDING`, `PACKAGE_INVENTORY_UNAVAILABLE`, events `package.operation.approved.v1`, `package.operation.dispatched.v1`, `package.operation.settled.v1`, `package.operation.reconciled.v1`, and tables `litespm_service_observations`, `package_operation_approvals`, `package_operation_receipts`, each labelled "accepted target interface (not yet in source)" until its implementing task lands. Decide O-03 in API.md.
- **Acceptance:**
  - [ ] Each name defined once in its owner doc and machine schema; nothing duplicated in this package.
  - [ ] Migration is the next immutable `sqlite-vNN.sql`; earlier migrations byte-identical; fingerprint test passes.
  - [ ] No table stores inventory items, catalog rows or host config.
  - [ ] Negative: schema rejects an approval without `plan_digest` or `expires_at`.
- **Command:** `python3 .specify/scripts/validate_feature_package.py` and `python3 scripts/validate_architecture.py` and `cargo test -p storage-sqlite --locked schema`
- **Evidence:** PR link and command output in Completion evidence.
- **Stop:** a promoted name collides with an owner definition; resolve at the owner.

## 013-T04 - LiteSpmClient: transport, handshake, version negotiation and peer authentication

- **Status:** BLOCKED (owner acceptance, T01). Nothing exists; reusable framing in `crates/operator-ipc`, SID helper in `crates/windows-local-security`.
- **Depends on:** T01, T02, T03
- **Cross-package prerequisites:** Feature 011 local IPC peer-identity profile (Wave 0).
- **Owner:** Capability Fabric, Security.
- **Allowed paths:** `apps/litecoworkd/src/litespm/mod.rs` (new), `apps/litecoworkd/src/litespm/client.rs` (new), `apps/litecoworkd/src/litespm/descriptor.rs` (new), `apps/litecoworkd/src/litespm/transport.rs` (new), `apps/litecoworkd/src/litespm/client_tests.rs` (new), `apps/litecoworkd/src/main.rs` (module declaration only), `apps/litecoworkd/Cargo.toml`.
- **Contract:** SPEC B-36, sections 7.2, 10, 12; AC-001, AC-002, AC-003, AC-016, AC-017.
- **Change intent:** implement `LiteSpmClient::connect` (async; `tokio::net::windows::named_pipe::ClientOptions` on Windows, `tokio::net::UnixStream` on Linux), `ProtocolPin`, `LiteSpmServiceDescriptor`, `PeerAuthObservation`; verify endpoint owner (pipe server process SID vs `current_logon_sid`, socket uid and `SO_PEERCRED`); cap every response with an `InFlightBodyBudget` before JSON parse; wrap every call in `tokio::time::timeout`; return `LiteSpmError::Unknown` (not `Failed`) when a mutating call times out after the request was written.
- **Acceptance:**
  - [ ] Fixture descriptors: pinned version accepted; other versions and missing descriptor refused (AC-001).
  - [ ] Windows `peerAuth.verified=false` recorded `TRANSPORT_ACL_ONLY`; foreign-owner endpoint refused (AC-002).
  - [ ] Client spawns no process and has no HTTP dependency (AC-003).
  - [ ] Oversized and malformed responses fail with typed errors; deadlines fire (AC-016).
  - [ ] Debug/Serialize of requests never print approval tokens (AC-017).
- **Command:** `cargo test -p litecoworkd --locked litespm_client` (filter to be added by this task)
- **Evidence:** test output; IMPLEMENTATION-STATUS.md LiteSPM row moves to CODE (client only).
- **Stop:** pinned protocol needs a capability the OS boundary cannot provide; return to owner.

## 013-T05 - Host compatibility profile and advertised-operation gating

- **Status:** BLOCKED (owner acceptance, T01; B-37).
- **Depends on:** T01, T04
- **Cross-package prerequisites:** Feature 002 exact agent identity and version for OpenCode v2.0.32 (binding creatable).
- **Owner:** Capability Fabric, Agent Runtime.
- **Allowed paths:** `apps/litecoworkd/src/litespm/host_profile.rs` (new), `apps/litecoworkd/src/litespm/mod.rs`, adjacent tests.
- **Contract:** SPEC B-37, section 12; AC-006.
- **Change intent:** add `HostCompatibilityProfile { host, host_version, os_build, scope, kind, operation, evidence_ref, qualified }` as compiled data, one row per B-37 pair and verb; `profile_allows(&descriptor, &profile, operation)` returns the operation only when LiteSPM advertises it and the row is qualified.
- **Acceptance:**
  - [ ] Skill install on OpenCode v2.0.32 user scope allowed only when advertised and qualified (AC-006).
  - [ ] Project scope, `mcp` install and unknown host versions return `PACKAGE_HOST_PROFILE_UNSUPPORTED` (AC-006).
  - [ ] Linux row disabled until T18 records its evidence.
- **Command:** `cargo test -p litecoworkd --locked litespm_host_profile` (filter to be added by this task)
- **Evidence:** profile table in this package PLAN section 4 updated with evidence refs.
- **Stop:** LiteSPM host adapter does not support the pair; owner chooses per B-37 fallback.

## 013-T06 - Live inventory read-through and Discover projection

- **Status:** BLOCKED (owner acceptance, T01). PARTIAL foundation: Discover search/describe and `DiscoverProviderStatus` exist in `crates/storage-sqlite/src/discover.rs`.
- **Depends on:** T01, T04, T05
- **Cross-package prerequisites:** NONE
- **Owner:** Capability Fabric, Operator.
- **Allowed paths:** `apps/litecoworkd/src/package_lifecycle_operator.rs` (new), `apps/litecoworkd/src/package_lifecycle_service.rs` (new), `apps/litecoworkd/src/operator.rs` (route merge only), `crates/storage-sqlite/src/discover.rs`, `crates/storage-core/src/discover.rs`, adjacent tests.
- **Contract:** SPEC sections 4, 7.2; AC-004, AC-005, AC-015, AC-016.
- **Change intent:** add `GET /v1/package-service/status` and `GET /v1/package-inventory`; map LiteSPM items to `DiscoverCatalogEntry` with `DiscoverEntrySource::LiteSpmPackage`, `authoritative: false` and the observed fingerprint; derive `DiscoverProviderStatus.package_service_qualified` from the live handshake instead of the hard-coded default. No inventory persistence.
- **Acceptance:**
  - [ ] Inventory page carries host, scope, locator, provenance, management state, fingerprint and observed time (AC-004).
  - [ ] LiteSPM read failure returns `PACKAGE_INVENTORY_UNAVAILABLE`, never an empty page (AC-004).
  - [ ] Same display name across scopes stays two rows (AC-005).
  - [ ] Availability, Connection auth, assignment and permission are separate fields (AC-015).
  - [ ] Page cap 200 and 4 MiB enforced; owner and Workspace fenced; foreign Runtime refused.
- **Command:** `cargo test -p litecoworkd --locked package_inventory` (filter to be added by this task) and `cargo test -p storage-sqlite --locked discover`
- **Evidence:** test names; served-route inventory regenerated in T19.
- **Stop:** any path writes inventory into SQLite.

## 013-T07 - Approval and operation-receipt storage with CAS and idempotency

- **Status:** BLOCKED (owner acceptance, T01).
- **Depends on:** T01, T03
- **Cross-package prerequisites:** Feature 005 Trust/approval primitive review (approval is owner-only, single-use).
- **Owner:** Storage, Trust.
- **Allowed paths:** `crates/storage-core/src/package_operation.rs` (new), `crates/storage-core/src/lib.rs`, `crates/storage-sqlite/src/package_operations.rs` (new), `crates/storage-sqlite/src/package_operation_tests.rs` (new), `crates/storage-sqlite/src/lib.rs`.
- **Contract:** SPEC sections 7.2, 8; DATA-MODEL sections 2-4; AC-007, AC-008.
- **Change intent:** implement `PackageOperationStore` (`record_approval`, `begin_apply`, `settle_apply`, `list_unsettled_package_operations`) with IMMEDIATE transactions; `begin_apply` consumes the approval with CAS (`state = GRANTED AND version = ?`) and inserts the `DISPATCHING` receipt and event in one transaction; unique index on `(approval_id)` for receipts and on `(workspace_id, host_profile_id, scope)` WHERE state IN (`DISPATCHING`, `UNKNOWN`) to block a second mutation per target.
- **Acceptance:**
  - [ ] Approval binds owner, digest, host profile, scope, expiry; consumed or expired approval refused (AC-007).
  - [ ] Two concurrent `begin_apply` calls: exactly one succeeds (AC-008).
  - [ ] Exact replay returns stored receipt; changed payload under same key is `IDEMPOTENCY_CONFLICT`.
  - [ ] Fault injection after each insert rolls back all rows.
- **Command:** `cargo test -p storage-sqlite --locked package_operation` (filter to be added by this task)
- **Evidence:** test output; DATA-MODEL updated with final column names.
- **Stop:** design requires storing LiteSPM-owned state beyond refs and receipts.

## 013-T08 - Plan and approval routes with owner-only authorization

- **Status:** BLOCKED (owner acceptance, T01).
- **Depends on:** T01, T05, T06, T07
- **Cross-package prerequisites:** NONE
- **Owner:** Operator, Trust, Capability Fabric.
- **Allowed paths:** `apps/litecoworkd/src/package_lifecycle_operator.rs`, `apps/litecoworkd/src/package_lifecycle_service.rs`, `apps/litecoworkd/src/package_lifecycle_operator_tests.rs` (new).
- **Contract:** SPEC sections 6 (steps 2-3), 7.2, 10; AC-006, AC-007.
- **Change intent:** `POST /v1/package-operations/plans` calls the host profile then `LiteSpmClient::plan`, returns digest, diff, hazards and expiry; `POST /v1/package-operations/approvals` accepts only an owner desktop principal and the exact digest returned by a plan for the same Workspace, host profile and scope. Requests carrying an agent, Conversation, Task or Coworker origin are refused.
- **Acceptance:**
  - [ ] Plan causes no mutation (fake client records no apply) (AC-006).
  - [ ] Agent-origin approval refused; foreign owner refused; expired or changed plan refused (AC-007).
  - [ ] Idempotency-Key required; replay returns the same approval.
- **Command:** `cargo test -p litecoworkd --locked package_plan_approval` (filter to be added by this task)
- **Evidence:** test output.
- **Stop:** pinned protocol cannot bind approval to digest; keep apply unavailable.

## 013-T09 - Apply, receipt, read-back and UNKNOWN reconciliation with startup sweep

- **Status:** BLOCKED (owner acceptance, T01).
- **Depends on:** T01, T07, T08
- **Cross-package prerequisites:** Feature 011 Runtime startup recovery hook (`recover_and_persist_startup_blockers` in `apps/litecoworkd/src/runtime.rs`).
- **Owner:** Capability Fabric, Storage, Runtime.
- **Allowed paths:** `apps/litecoworkd/src/package_lifecycle_service.rs`, `apps/litecoworkd/src/package_lifecycle_operator.rs`, `apps/litecoworkd/src/runtime.rs` (one call into the package reconciler), adjacent tests.
- **Contract:** SPEC section 6 steps 4-6, section 11; AC-008, AC-009, AC-010.
- **Change intent:** `apply` = `begin_apply` (transaction) -> `LiteSpmClient::apply` (outside) -> `settle_apply` (transaction) with LiteSPM operation ref and read-back fingerprint; timeout after write settles `UNKNOWN`; `reconcile` and the startup sweep call `operation_get` for every `DISPATCHING`/`UNKNOWN` receipt; a `DISPATCHING` row with no evidence the request was written is reconciled through `operation_get` by apply key, never re-sent blindly.
- **Acceptance:**
  - [ ] Success visible only after receipt and read-back; inventory refreshed (AC-010).
  - [ ] Lost response produces UNKNOWN and blocks further mutation for that target (AC-009).
  - [ ] Restart resumes reconciliation; no second apply (AC-009).
  - [ ] Duplicate apply key dispatches once (AC-008).
  - [ ] No assignment, grant or invocation record is created (AC-010).
- **Command:** `cargo test -p litecoworkd --locked package_apply` (filter to be added by this task)
- **Evidence:** test output; fault-injection log.
- **Stop:** LiteSPM offers no operation query; apply stays unavailable (B-36).

## 013-T10 - Adopt and unmanage (observe-first)

- **Status:** BLOCKED (owner acceptance, T01; upstream unmanage absent at `f77a3a3`, O-06).
- **Depends on:** T01, T09
- **Cross-package prerequisites:** NONE
- **Owner:** Capability Fabric.
- **Allowed paths:** `apps/litecoworkd/src/package_lifecycle_service.rs`, `apps/litecoworkd/src/litespm/host_profile.rs`, adjacent tests.
- **Contract:** SPEC G-08, L-06; AC-011.
- **Change intent:** enable `Adopt` and `Unmanage` plan kinds only when advertised; require LiteSPM's plan to declare an ownership-record-only change for adopt; present keep-local as default on drift.
- **Acceptance:**
  - [ ] Adopt leaves the host skill directory byte-identical (fixture hash before/after) (AC-011).
  - [ ] Unmanage preserves the originally present entry (AC-011).
  - [ ] Drifted entry defaults to keep-local; stale fingerprint refuses the plan.
- **Command:** `cargo test -p litecoworkd --locked package_adopt` (filter to be added by this task)
- **Evidence:** fixture hashes in test output.
- **Stop:** any path deletes an originally present native entry.

## 013-T11 - Rollback, remove, update and repair

- **Status:** BLOCKED (owner acceptance, T01).
- **Depends on:** T01, T09, T10
- **Cross-package prerequisites:** NONE
- **Owner:** Capability Fabric.
- **Allowed paths:** `apps/litecoworkd/src/package_lifecycle_service.rs`, adjacent tests.
- **Contract:** SPEC G-07, G-08; AC-012, AC-013.
- **Change intent:** rollback after a failed apply requests a LiteSPM remove plan for owned nodes only and needs its own owner approval; update and repair always plan against the current fingerprint; a changed permission or tool-schema fingerprint emits the notification Feature 006 consumes.
- **Acceptance:**
  - [ ] Remove deletes only unchanged LiteSPM-owned nodes (LiteSPM read-back) (AC-012).
  - [ ] Daemon holds no host-path write call (static test) (AC-012).
  - [ ] Update with a stale fingerprint is refused; schema change triggers notification (AC-013).
- **Command:** `cargo test -p litecoworkd --locked package_remove_update` (filter to be added by this task)
- **Evidence:** test output.
- **Stop:** LiteSPM rollback semantics are unproven on the pin; keep update/repair unavailable.

## 013-T12 - Tauri package lifecycle bridge commands

- **Status:** BLOCKED (owner acceptance, T01).
- **Depends on:** T01, T06, T08, T09
- **Cross-package prerequisites:** Feature 011 packaged sidecar and Windows named-pipe IPC.
- **Owner:** Desktop bridge.
- **Allowed paths:** `apps/litecowork-ui/src-tauri/src/package_lifecycle_bridge.rs` (new), `apps/litecowork-ui/src-tauri/src/lib.rs` (command registration only).
- **Contract:** SPEC section 7.2; AC-014, AC-017.
- **Change intent:** add `package_service_status`, `package_inventory_request`, `package_plan_request`, `package_approval_request`, `package_apply_request`, `package_operation_get`, `package_operation_reconcile`, each validating ids, bounding query and response size like `discover_bridge.rs`, generating the Idempotency-Key once per user intent and never retrying a mutating call with a new key.
- **Acceptance:**
  - [ ] Inputs bounded and path components encoded (unit tests like `item_refs_are_encoded_as_one_path_component`).
  - [ ] No approval token or package content in bridge logs (AC-017).
- **Command:** `cargo test -p litecowork-ui --locked package_lifecycle_bridge` (filter to be added by this task)
- **Evidence:** test output.
- **Stop:** bridge needs to call LiteSPM directly; forbidden.

## 013-T13 - Discover UI: Installed / native, Managed, plan review and lifecycle verbs

- **Status:** BLOCKED (owner acceptance, T01). PARTIAL foundation: tabs and truthful unavailable states in `DiscoverPage.tsx`; `connectActionOutcome()` in `discover-state.ts`.
- **Depends on:** T01, T12
- **Cross-package prerequisites:** Feature 011 app-wide keyboard and focus baseline.
- **Owner:** Desktop UI.
- **Allowed paths:** `apps/litecowork-ui/src/discover/DiscoverPage.tsx`, `apps/litecowork-ui/src/discover/discover-state.ts`, `apps/litecowork-ui/src/discover/discover-api.ts`, `apps/litecowork-ui/src/discover/discover-page.css`, `apps/litecowork-ui/src/discover/package-lifecycle-api.ts` (new), `apps/litecowork-ui/src/discover/package-lifecycle-state.ts` (new), `apps/litecowork-ui/src/discover/PackagePlanReview.tsx` (new), `apps/litecowork-ui/tests/package-lifecycle-state.test.ts` (new), `apps/litecowork-ui/tests/discover-catalog-state.test.ts`.
- **Contract:** SPEC sections 6, 14; contracts/ui-behavior.md; AC-014, AC-015.
- **Change intent:** render LiteSPM inventory in "Installed / native" and Managed; a pure reducer in `package-lifecycle-state.ts` drives states `LOADING`, `EMPTY`, `UNAVAILABLE`, `OFFLINE`, `STALE`, `PLANNING`, `AWAITING_APPROVAL`, `APPLYING`, `UNKNOWN`, `SUCCEEDED`, `FAILED`; `PackagePlanReview` shows the plain-text diff, host, scope, expiry and hazards with Approve/Cancel; each verb is its own control; unavailable verbs disabled with `aria-describedby` reason; Escape cancels the dialog and focus returns to the invoking control.
- **Acceptance:**
  - [ ] Every state above renders distinct text and role (status/alert) (AC-014).
  - [ ] No irreversible action before diff confirmation; Approve disabled after expiry (AC-014).
  - [ ] Four facts shown separately; `authoritative` stays false (AC-015).
  - [ ] Keyboard-only path through plan, approve, progress; screen-reader names for every verb.
  - [ ] Package text rendered as plain text only.
- **Command:** `pnpm --dir apps/litecowork-ui test` and `pnpm --dir apps/litecowork-ui run build`
- **Evidence:** test output; UI ledger rows updated in T19.
- **Stop:** a control would render enabled without an owner route.

## 013-T14 - Capability bridge input producer and Feature 006 handoff

- **Status:** BLOCKED (owner acceptance, T01). PARTIAL foundation: `classify_agent_capability_route` exists with tests and no caller.
- **Depends on:** T01, T09
- **Cross-package prerequisites:** Feature 006 assignment/grant model and grant-invalidation consumer.
- **Owner:** Capability Fabric, Feature 006 owner.
- **Allowed paths:** `apps/litecoworkd/src/package_lifecycle_service.rs`, `crates/domain-invocations/src/capability_bridge.rs` (tests only), adjacent tests.
- **Contract:** SPEC G-09, sections 10, 14; AC-010, AC-013, AC-018.
- **Change intent:** build `AgentCapabilityBridgeInput` identity fields (`capability_identity_digest`, `lite_spm_managed`, `package_digest_verified`) only from a settled receipt matched to current inventory by LiteSPM identity and digest; leave every `direct_path_*` field false until Feature 006/002 qualify them; expose "installed, not granted" to Discover and hand assignment/grant to Feature 006.
- **Acceptance:**
  - [ ] Same-named native entry without a receipt yields `PACKAGE_IDENTITY_UNQUALIFIED` (AC-018).
  - [ ] Installed package creates no grant (AC-010).
  - [ ] Fingerprint change emits the invalidation notification (AC-013).
- **Command:** `cargo test -p domain-invocations --locked capability_bridge` and `cargo test -p litecoworkd --locked package_capability_identity` (filter to be added by this task)
- **Evidence:** test output.
- **Stop:** identity would be derived from display name.

## 013-T15 - Discover truthfulness fixes (timestamps, fail-closed connect-or-install)

- **Status:** BLOCKED (owner acceptance, T01). PARTIAL: fail-closed 503 already correct; timestamp defect present in `discover_operator.rs`.
- **Depends on:** T01
- **Cross-package prerequisites:** NONE
- **Owner:** Operator.
- **Allowed paths:** `apps/litecoworkd/src/discover_operator.rs`, `apps/litecoworkd/src/routine_run_operator_tests.rs` (the existing capability test only).
- **Contract:** SPEC section 4; AC-019, AC-020.
- **Change intent:** replace `new_id("cor")` used as `checked_at` and as `now` with the daemon's RFC 3339 clock helper; keep `connect_or_install_capability` returning 503 `LITESPM_PACKAGE_SERVICE_NOT_QUALIFIED` until T09 and T18 pass on a row, then apply the O-03 decision.
- **Acceptance:**
  - [ ] `checked_at` parses as RFC 3339 (AC-020).
  - [ ] connect-or-install still writes no receipt and grants no scope (AC-019).
- **Command:** `cargo test -p litecoworkd --locked capability_commands_require_an_idempotency_key_and_stay_truthful`
- **Evidence:** test output.
- **Stop:** NONE beyond test failure.

## 013-T16 - Failure, crash and race matrix; create the canonical test target

- **Status:** BLOCKED (owner acceptance, T01).
- **Depends on:** T01, T09-T11, T14
- **Cross-package prerequisites:** NONE
- **Owner:** Capability Fabric, Testing.
- **Allowed paths:** `apps/litecoworkd/tests/litespm_package_lifecycle.rs` (new), `apps/litecoworkd/tests/support/fake_litespm.rs` (new).
- **Contract:** SPEC section 11; contracts/errors-recovery.md; AC-008, AC-009, AC-012.
- **Change intent:** a fake LiteSPM daemon over a real local socket/pipe that can drop responses, delay past deadlines, return oversized or malformed frames, restart, change fingerprints and report drift; scenarios: kill daemon after `begin_apply`, after request write, before `settle_apply`; LiteSPM restart mid-apply; duplicate approval replay; concurrent apply; plan expiry during approval; inventory failure.
- **Acceptance:**
  - [ ] Every scenario ends with exactly one LiteSPM apply or none, and a terminal or reconciled receipt (AC-008, AC-009).
  - [ ] No scenario reports success without receipt and read-back.
  - [ ] Rollback path never writes host files (AC-012).
- **Command:** `cargo test -p litecoworkd --test litespm_package_lifecycle --locked -- --nocapture`
- **Evidence:** test output in Completion evidence.
- **Stop:** a scenario dispatches twice.

## 013-T17 - Cross-layer trace for every user-visible verb (AGENTS.md section 4)

- **Status:** BLOCKED (owner acceptance, T01).
- **Depends on:** T01, T13, T16
- **Cross-package prerequisites:** NONE
- **Owner:** Capability Fabric, Desktop UI.
- **Allowed paths:** this package (`contracts/ui-behavior.md`, `WIRING.yaml`).
- **Contract:** AGENTS.md section 4; AC-004, AC-010, AC-014.
- **Change intent:** for View inventory, Install to agent, Manage with LiteSPM, Stop managing, Remove installation, Update, Repair and Reconcile, record control -> Tauri command -> route -> authorization -> service -> transaction/fence -> receipt/event -> projection -> restart behaviour -> test name.
- **Acceptance:**
  - [ ] Every verb has a complete row with a real test name; no route without a control and no control without a route.
- **Command:** `python3 .specify/scripts/validate_feature_package.py`
- **Evidence:** trace table in contracts/ui-behavior.md.
- **Stop:** a verb has no owner route; disable it in UI.

## 013-T18 - Qualification on the release rows (E2E-06, LSI rows)

- **Status:** BLOCKED (owner acceptance, T01; pinned LiteSPM build available on the release machines).
- **Depends on:** T01, T13-T17
- **Cross-package prerequisites:** Feature 019 release matrix rows and evidence bundle process; Feature 006 grant/use for the E2E-06 tail; Feature 002 OpenCode v2.0.32 binding.
- **Owner:** Product, Testing, Capability Fabric.
- **Allowed paths:** this package; evidence report path chosen by Feature 019.
- **Contract:** ACCEPTANCE-AND-E2E-PLAN E2E-06; LITESPM-INTEGRATION-PLAN section 9 (LSI-012, LSI-014, LSI-017 to LSI-030, LSI-034, LSI-035); AC-002, AC-011, AC-016.
- **Change intent:** run CODE, SYSTEM and owner USER evidence on Windows 11/10 x64 + OpenCode v2.0.32 (skill, user scope) then Linux x64: discover existing skill, adopt with byte-identical config, install a new skill, kill during apply and recover, unmanage, remove, then hand to Feature 006 for grant/use; include denial (agent self-approval, expired plan) and recovery.
- **Acceptance:**
  - [ ] Packaged desktop E2E-06 passes on row 1 with owner USER sign-off, then row 2.
  - [ ] Peer-auth class recorded per row (AC-002); adopt byte-identical on the real host (AC-011); p95 budget measured (AC-016).
  - [ ] LSI rows listed above pass or are recorded UNAVAILABLE with reason.
- **Command:** `cargo test -p litecoworkd --test litespm_package_lifecycle --locked -- --nocapture` and `python3 scripts/v1_platform_matrix.py`
- **Evidence:** Feature 019 evidence bundle; IMPLEMENTATION-STATUS.md row per release row.
- **Stop:** USER evidence missing on a row; that row stays unqualified and verbs stay disabled there.

## 013-T19 - Documentation sync

- **Status:** BLOCKED (owner acceptance, T01).
- **Depends on:** T01, T18
- **Cross-package prerequisites:** orchestrator owns register, specs/README.md and traceability index.
- **Owner:** Documentation owners.
- **Allowed paths:** `docs/IMPLEMENTATION-STATUS.md`, `docs/schemas/served-route-inventory.json`, `docs/V1-UI-ACTION-STATE-LEDGER.md`, `docs/LITESPM-LITECOWORK-INTEROP-LLD.md`, this package.
- **Contract:** AGENTS.md sections 4-5; all AC IDs.
- **Change intent:** update the "Discover and LiteSPM" status rows with exact evidence class per row; regenerate the served-route inventory; update UI ledger rows for Discover verbs; close B-36/B-37 register rows (orchestrator).
- **Acceptance:**
  - [ ] Served-route inventory lists the new routes from source.
  - [ ] Status text never claims more than the recorded evidence.
- **Command:** `python3 scripts/audit_served_routes.py` and `python3 .specify/scripts/validate_feature_package.py`
- **Evidence:** diff and command output.
- **Stop:** a status claim lacks evidence.

## 013-T20 - Pin the LiteSPM bridge and mirror its safety contract (P-17)

- **Status:** NOT STARTED - added under this package's IN_REVIEW status. Folds proposed task P-17 from docs/REFERENCE-SYNTHESIS-AND-V1-FEATURE-ALIGNMENT-2026-11.md section 8 (and Section 2) into this package.
- **Depends on:** T04
- **Cross-package prerequisites:** NONE
- **Owner:** Package Lifecycle, Connections, Storage.
- **Allowed paths:** `specs/013-litespm-package-lifecycle-bridge/RESEARCH.md`, `apps/litecoworkd/src/`, adjacent tests.
- **Contract:** SPEC sections 5-7; AC-018, AC-021; AGENTS.md section 6; RESEARCH pinned sources.
- **Change intent:** pin the LiteSPM bridge from its published package and protocol, reconciling the three observed SHAs 9f42b25d, 111fd9b8 and f77a3a3; mirror the LiteSPM preview, backup, transaction and byte-exact-restore safety contract so a package mutation is previewed, backed up and either committed atomically or restored byte-for-byte; and adopt single-flight plus backoff for Discover read-through. Wire names, version and authentication come only from the published pinned bridge contract (LiteSPM publishes it at `ARCH/06-API-CONTRACTS.md` and `ARCH/11-LOCAL-RUNTIME-IPC.md`), never inferred from CLI output or a marketplace listing (AGENTS.md section 6). RESEARCH.md records the pin as TARGET-pending-published-bridge because the pin source is LiteSPM's own published package or protocol release, which must be confirmed against that published contract at implementation time; reading a reference clone during research does not make its checkout the authoritative pin.
- **Acceptance:**
  - [ ] RESEARCH.md records the pin with the three reconciled SHAs and the TARGET-pending-published-bridge status, and the exact wire names come only from the published bridge contract (AC-021).
  - [ ] The preview, backup, transaction and byte-exact-restore safety contract is mirrored; a failed mutation restores byte-for-byte (AC-021).
  - [ ] Discover read-through uses single-flight plus backoff; the receipt-to-inventory match stays digest-bound (AC-018).
- **Command:** `python3 .specify/scripts/validate_feature_package.py`
- **Evidence:** RESEARCH.md pin note and single-flight test output in Completion evidence.
- **Stop:** the published bridge contract cannot be fetched; keep the pin TARGET-pending-published-bridge and do not infer wire names.

## 013-T21 - Package VERIFIED for V1

- **Status:** BLOCKED (owner acceptance, T01).
- **Depends on:** T01-T20
- **Cross-package prerequisites:** Feature 019 release sign-off process.
- **Owner:** Product (repository owner).
- **Allowed paths:** this package; `docs/IMPLEMENTATION-STATUS.md`.
- **Contract:** SPEC sections 14-15; AC-001 to AC-021; SPECIFICATION-STANDARD lifecycle.
- **Change intent:** assemble the evidence bundle; move IN_REVIEW -> ACCEPTED/LOCKED (T01) -> IMPLEMENTING (first implementation merge) -> VERIFIED after owner sign-off on both rows.
- **Acceptance:**
  - [ ] Canonical command passes on both rows against the pinned LiteSPM build.
  - [ ] Owner USER sign-off for E2E-06 on each row, including denial and recovery.
  - [ ] SPEC and WIRING status VERIFIED; validator passes.
- **Command:** `cargo test -p litecoworkd --test litespm_package_lifecycle --locked -- --nocapture` and `python3 .specify/scripts/validate_feature_package.py`
- **Evidence:** evidence bundle path referenced from IMPLEMENTATION-STATUS.md.
- **Stop:** any row lacks USER evidence.

## Implementation and optimization notes (Rust and TypeScript)

Findings from reading the source. They are folded into the tasks above and add no scope.

- **Correlation ID used as a timestamp (Rust, T15):** `test_capability_connection` returns `checked_at: new_id("cor")` and `search_discover` passes a correlation ID as `now` to `search_catalog`. Any freshness or expiry logic built on that value would be wrong; use one clock helper.
- **Async client, no blocking pool (Rust, T04):** Discover handlers already push SQLite work to `spawn_blocking`. The LiteSPM client should be fully async (tokio named pipe / `UnixStream`) with `tokio::time::timeout`, so a slow LiteSPM never occupies a blocking thread and deadlines are cheap.
- **Reuse bounded framing (Rust, T04):** `crates/operator-ipc` already has `InFlightBodyBudget` and `BudgetedBody`. Reusing them caps memory before JSON parse instead of reading an unbounded body into `Vec<u8>`.
- **Database-level single mutation per target (Rust/SQLite, T07):** a partial unique index on receipts in `DISPATCHING`/`UNKNOWN` per `(workspace_id, host_profile_id, scope)` enforces "UNKNOWN blocks further mutation" in one index lookup, without an application lock.
- **No inventory persistence (Rust, T06):** read-through with an optional per-request page cache keyed by observed fingerprint keeps LiteCowork from becoming a second package DB and avoids cache invalidation bugs.
- **Pure UI reducer (TypeScript, T13):** `DiscoverPage.tsx` keeps request generations and abort controllers inline. A pure `package-lifecycle-state.ts` reducer is unit-testable with the existing `node --test` runner. Authority (approval, digest binding, idempotency, reconciliation) stays in the Rust daemon, never in the WebView.

## Dependency order

T01 -> T02 -> T03 -> T04 -> T05 -> T06. T03 -> T07. (T05, T06, T07) -> T08 -> T09 -> T10 -> T11. T09 -> T14. (T06, T08, T09) -> T12 -> T13. T01 -> T15. (T09-T11, T14) -> T16. (T13, T16) -> T17. (T13-T17) -> T18 -> T19. T04 -> T20 (LiteSPM bridge pin, P-17). (T19, T20) -> T21 (VERIFIED).

## Cross-package dependencies

| This task | Other package task | Reason |
|---|---|---|
| T04 | Feature 011 local IPC peer-identity profile (Wave 0) | Same OS peer-identity rules for the second IPC boundary |
| T05, T18 | Feature 002 OpenCode v2.0.32 binding creatable (probe fix) | Host profile row needs exact agent identity and version |
| T07 | Feature 005 Trust/approval primitives | Approval semantics must not fork from Trust |
| T03, T11, T14 | Feature 006 Connection/assignment/grant and grant invalidation | Installed is not granted; 006 consumes the fingerprint notification |
| T09 | Feature 011 Runtime startup recovery hook | Startup sweep for UNKNOWN receipts |
| T12 | Feature 011 packaged sidecar and Windows named pipe | Desktop path for E2E-06 |
| T13 | Feature 011 keyboard and focus baseline | Shared accessibility rules |
| T18, T20 | Feature 019 release matrix rows and evidence bundle | Qualification is per release row |

Cross-package task IDs are named by outcome because other packages are being renumbered in the same planning pass; the orchestrator resolves them in the traceability index. Waves: observe and inventory (T02-T06, T13 read-only) in Wave 2; plan/apply/receipt (T07-T12, T14) in Wave 3; qualification in Wave 3 to 6.

## Completion evidence

For each task append a row when it closes: task, code revision (SHA), exact command, result (pass/fail with counts), OS and build (Windows 11/10 x64 or Linux x64 distribution), LiteSPM pin, host and version (OpenCode v2.0.32), evidence class (CODE, SYSTEM, USER), report path and reviewer. Statuses above are source-audit classifications at this revision, not qualification.

| Task | Revision | Command | Result | OS/build | LiteSPM pin / host | Class | Reviewer |
|---|---|---|---|---|---|---|---|
