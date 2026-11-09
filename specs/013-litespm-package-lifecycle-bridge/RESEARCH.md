# Research and Decision Log - LiteSPM Package Lifecycle Bridge

**Feature:** 013 | **Status:** IN_REVIEW - not owner-accepted; does not authorize implementation; not a claim of implementation or qualification.
**Checked:** 2026-11-09 - repository source and docs read; LiteSPM upstream read read-only at HEAD 84d5fab (v0.6.0) for the integration study; not built or run for this package; no web research.

## 1. Pinned external sources

| Source | Revision | License | Finding | Decision |
|---|---|---|---|---|
| REF-LITESPM [sarv-projects/LiteSPM](https://github.com/sarv-projects/LiteSPM) | `9f42b25ddfb1acedce0f2dbf9964f4a7724d622e` ([REFERENCE-IMPLEMENTATIONS.md](../../docs/REFERENCE-IMPLEMENTATIONS.md)) | Apache-2.0 at root LICENSE | Selected lifecycle authority; "its current checkout does not settle the LiteCowork-facing package API" | Prototype against its published contract only; copy no code; reject inferred wire names |
| LiteSPM at interop LLD header pin | `111fd9b8e84573f8fce76b6374b4ff9639502539` | Apache-2.0 | Upstream STATUS marks the 13-tool stdio bridge WIRED; Windows named pipe owner-only DACL with `peerAuth.verified=false` | Agent-facing ABI only; not this client's transport |
| LiteSPM at interop LLD Appendix A/B study | `f77a3a3eeb38396788cc31240105cf2e0321a205` | Apache-2.0 | 20 daemon methods + `daemon.handshake`; approvals recorded only by the human CLI (A1); no package `operation.get` (A2); no reconcile/unmanage (A3); no service descriptor (A4); health meaning unagreed (A5); unsigned catalog pointer (A6). Semantic-to-wire mapping recorded in Appendix B4 | Use only after re-pin at 013-T02 confirms the same at the chosen revision |

The three pins disagree. 013-T02 chooses one, records it here and in both LiteSPM owner docs, and marks the others historical.

## 2. Repository sources inspected

| Source | Path | Finding |
|---|---|---|
| Discover routes | `apps/litecoworkd/src/discover_operator.rs` | Search/describe served; connect-or-install always 503 `LITESPM_PACKAGE_SERVICE_NOT_QUALIFIED`; test returns UNKNOWN; correlation ID used as `checked_at` and `now` |
| Discover projection | `crates/storage-sqlite/src/discover.rs`, `crates/storage-core/src/discover.rs` | Skills + 50 host observations; `authoritative` always false; provider status hard-coded |
| Route classifier | `crates/domain-invocations/src/capability_bridge.rs` | Requires LiteSPM-managed + digest-verified identity; no caller |
| Desktop Discover | `apps/litecowork-ui/src/discover/*`, `apps/litecowork-ui/src-tauri/src/discover_bridge.rs` | Four tabs; Managed/Connections unavailable; all actions disabled; bridge supports search/item only |
| IPC primitives | `crates/operator-ipc/src/lib.rs`, `crates/windows-local-security/src/lib.rs` | Bounded body budgets; logon SID helper; pipe connect helper |
| Owner prose | `docs/LITESPM-LITECOWORK-INTEROP-LLD.md`, `docs/LITESPM-INTEGRATION-PLAN.md`, `docs/IMPLEMENTATION-MASTER-PLAN.md` section 3 and P5 | Ownership, verbs, plan/apply/recovery, LSI-001 to LSI-040, lane steps 1-7 |
| Flows | `docs/FLOWS.md` F04 (capability discovery and direct attachment), F22 (package update while Task running) | Upstream-owned update notification; Task does not restart for discovery |

## 3. Alternatives

| Alternative | Decision | Rationale |
|---|---|---|
| Parse `litespm list --json` or other CLI output | Reject | AGENTS.md section 6; CLI output is not a contract |
| Use the 13-tool MCP stdio bridge as LiteCowork's client | Reject | It is the agent-facing ABI; an agent path cannot carry owner approval |
| Spawn `litespm approve <plan-id>` | Reject | Interactive behaviour unproven; approval must be a typed, digest-bound daemon operation |
| Mirror LiteSPM inventory into LiteCowork SQLite | Reject | Second package DB (L-02) and stale-state risk |
| Daemon-local IPC with descriptor negotiation | Select (B-36) | Matches LITESPM-INTEGRATION-PLAN section 4 and upstream daemon design |
| First kind `mcp` | Defer | Launches a provider process; health meaning (A5) unagreed |
| First kind `skill` by copy | Select (B-37) | Upstream copy-install with provenance ledger and path confinement; no provider process or secrets |
| Separate crate for the client | Flexible (F-01) | Faster isolated tests vs one more workspace member |

## 4. Open research

| ID | Topic | Next evidence | Owner |
|---|---|---|---|
| R-01 | Published descriptor and approval operation at the chosen pin | Upstream schema files | LiteSPM owner |
| R-02 | OpenCode v2.0.32 skill directory support in LiteSPM host adapter | Upstream host table and a read-back fixture | LiteSPM owner, Agent Runtime |
| R-03 | Windows pipe server SID verification API and LiteSPM pipe name | Upstream IPC source at pin | Security |
| R-04 | Linux socket path and `SO_PEERCRED` behaviour under the LiteSPM daemon | Upstream IPC source at pin | Security |
| R-05 | Rollback semantics on partial skill copy | Upstream install/remove tests at pin | LiteSPM owner |

## 5. LiteSPM bridge pin (P-17, folded 2026-11-09)

Task 013-T20 folds proposed task P-17 from [REFERENCE-SYNTHESIS-AND-V1-FEATURE-ALIGNMENT-2026-11.md](../../docs/REFERENCE-SYNTHESIS-AND-V1-FEATURE-ALIGNMENT-2026-11.md) section 8. It pins the LiteSPM bridge from its published package and protocol, reconciling the three disagreeing SHAs recorded in section 1 (`9f42b25d`, `111fd9b8`, `f77a3a3`), and mirrors the LiteSPM preview, backup, transaction and byte-exact-restore safety contract, plus single-flight with backoff for Discover read-through.

Pin status (updated 2026-11-09): the LiteSPM repository was read at HEAD **`84d5faba75bc621a651e0cec0f831e936bb2082d`** (`git describe` `v0.6.0-143-g84d5fab`, npm `0.6.0`, Apache-2.0), and its published contract was transcribed into [LITESPM-INTEGRATION-STUDY-2026-11.md](../../docs/LITESPM-INTEGRATION-STUDY-2026-11.md). The three disagreeing SHAs below (`9f42b25d`, `111fd9b8`, `f77a3a3`) are superseded by this single published HEAD. The confirmed contract: JSON-RPC 2.0 line-delimited local IPC (16 MiB max), named pipe `\\.\pipe\litespm-daemon-<12 hex of SHA256(username)>` with owner-only DACL on Windows and `<RuntimeRoot>/litespm.sock` with `SO_PEERCRED`/`LOCAL_PEERCRED` peer-uid refusal on Linux/macOS; `daemon.handshake` plus 19 application methods (`catalog.search`, `catalog.get_item`, `resolver.prepare_plan`, `install.execute`, `install.remove`, `skills.*`, `capabilities.search`/`describe`, `provider.probe`/`invoke`, `invocation.get`/`cancel`, `host.detect_config`/`apply_setup`, `doctor.run_checks`, `system.status`); protocol version `2026-07-28`; `planHash` = `sha256:` of RFC 8785 canonical JSON of the plan execution fields, which LiteCowork approval binds to; a 17-action effect taxonomy and a central egress/SSRF/secret-vault model that matches LiteCowork invariants. The handshake is defined but not yet enforced by LiteSPM, so this client self-verifies `peerAuth` and `protocolVersion` and fails closed. The pin is still re-read at 013-T02 / 013-T20 implementation time and the exact release qualified against is recorded, because LiteSPM's release round trip can move HEAD and "a probe is evidence only for the release it names" (its own STATUS rule); reading the published source is the authoritative basis, not inference from CLI output or a marketplace listing (AGENTS.md section 6). LiteSPM publishes its contract (its API surface at `ARCH/06-API-CONTRACTS.md` and its local runtime IPC at `ARCH/11-LOCAL-RUNTIME-IPC.md`, plus `schemas/` and `ARCH/05-SECURITY.md`), and the exact wire names, version and authentication must be taken from that published package or protocol at 013-T02 / 013-T20 time, never inferred from CLI output or a marketplace listing (AGENTS.md section 6). The three disagreeing SHAs recorded in section 1 (`9f42b25d`, `111fd9b8`, `f77a3a3`) and the preview/backup/transaction/byte-exact-restore safety contract are reconciled against that published contract when the pin is taken; reading a reference clone during research does not make its checkout the pin. No LiteSPM code is copied into LiteCowork; `repo_take/` reference clones are research-only.
