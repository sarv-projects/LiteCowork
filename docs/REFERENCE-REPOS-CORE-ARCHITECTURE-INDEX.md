# Reference Repositories — Core Architecture Reading Index

**Status:** research procedure + pinned paths (not normative product)  
**Updated:** 2026-11-08 (Phase 2 rival re-verification: ZCode→3.17.2, DeepSeek Harness desktop→0.2.2-rc.1; added OpenWork, Open Cowork, Rowboat, OpenLoomi read-first indexes)  
**Companion:** [DESKTOP-REFERENCE-IMPLEMENTATION-AUDIT.md](DESKTOP-REFERENCE-IMPLEMENTATION-AUDIT.md) (ZCode + DeepSeek Harness desktop deep dive)

Reading **every line** of large repos is not required. Before borrowing behavior or tests, read the **core architecture, IPC, persistence, agent loop, and lifecycle** paths below at the pinned revision.

Local clones (gitignored, absent from a fresh checkout): `repo_take/` per `repo_take/README.md`.

---

## ZCode — `zai-org/ZCode`

| Pin | `a9fe401eede6e6e5e01be9048679c206fe6f02d9` (v3.17.2, 2026-11-08; prior pin `a60ba6a7…` was v3.17.1/2026-10-30) |
| App path | `packages/desktop` (not `apps/zcode-cli`) |
| License | Apache-2.0 + NOTICE |

### Read first (architecture)

| Path (under pin) | Why |
|---|---|
| `docs/architecture/zcode-code-architecture-overview.md` | Main / Renderer / Host / CLI ownership |
| `packages/desktop/src/main/index.ts` | Shell composition |
| `packages/desktop/src/main/desktopHostProcess.ts` | UtilityProcess Host |
| `packages/desktop/src/host/hostInitialization.ts`, `hostShutdownPhases.ts` | Partial start rollback, teardown |
| `packages/desktop/src/main/pluginSandbox/CONTRACT.md` | Guest sandbox rules |
| `packages/desktop/test/e2e/conversation-session/` | Session USER test themes |

### Dependencies / flows

- CLI owns conversation state; Host routes scopes; Renderer never calls CLI directly.  
- Compare to LiteCowork: **UI ≠ daemon**; Core SQLite truth.

---

## DeepSeek Harness — `deepseek-ai/deepseek-harness`

| Pin | `fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159` (desktop v0.2.2-rc.1, 2026-11-08; prior pin `ce7073d5…` was v0.2.2-alpha.2/2026-10-30) |
| App path | `apps/desktop` |
| License | MIT |

### Read first

| Path | Why |
|---|---|
| `apps/desktop/src/main.ts` | Window + lifecycle |
| `apps/desktop/src/ipc.ts` | Typed channels + sender assert |
| `apps/desktop/src/preload-app.ts` | contextBridge surface |
| `apps/desktop/src/host-process.ts` | Host lifecycle, deadlines |
| `apps/desktop/src/quit-confirmation.ts` | Active work UNKNOWN UX |
| Desktop tests under `apps/desktop` (ipc, preload, quit) | USER prompts for LiteCowork |

### Gap vs LiteCowork

Task/Effect drain on quit **not integrated** in LiteCowork owners — see audit §4.3.

---

## AionUi — `iOfficeAI/AionUi`

| Pin | `11cd2a9a20d1e033157636efc570b41384c66d85` (2026-11-01) |

### Read first

| Path | Why |
|---|---|
| `docs/contributing/file-structure.md` | process/renderer/common split |
| `packages/desktop/src/common/platform/bridge.ts` | invoke/emit typed bridge |
| `tests/unit/common-platform/bridge.test.ts` | Transport tests |
| `src/process/bridge/`, `src/process/task/`, `src/process/agent/` | IPC surface inventory |

---

## OpenDot — `athulsreekumar/opendot`

| Pin | `ec75891b819713acf29cb659bd9c72d1ed34f76a` (2026-11-01) |

### Read first

| Path | Why |
|---|---|
| Root README architecture diagram | Watchers, Dot Links, SuperDot |
| `desktop/` Electron app | Session UI + Pi integration |
| `~/.opendot` data model (documented) | Local-first audit/masking patterns |

**Not** OpenAI dots; **not** `itsperini/opendot` (voice AGPL).

---

## Eigent — `eigent-ai/eigent`

| Pin | `2ffa2e4ef19356e87479845f2aa1600bc2fde671` |

### Read first

| Path | Why |
|---|---|
| `backend/app/utils/workforce.py` | Decomposition/retry (reference only) |
| `src/components/Trigger/*` | Automation UI patterns |

---

## OpenWork — `different-ai/openwork`

| Pin | `5493b18a6af64d56454658a53da00eabbc6e033d` (2026-11-08) |
| App path | `apps/desktop` client + optional `apps/server` control plane; capabilities under `packages/*` |
| License | MIT except `/ee` (Enterprise Edition); pre-EE content FSL-1.1 / MIT-Future |

### Read first

| Path (under pin) | Why |
|---|---|
| `DESIGN.md` | Chat-first doctrine; compact tool activity; no auto side-panel on background result |
| `packages/automations/src/engine.ts`, `ports.ts`, `contracts.ts` | Durable automation receipts/events survive the admitting process exit — compare to AUTOMATION/TASK-RUNTIME |
| `packages/automations/src/calendar-trigger.ts` | Event/calendar trigger shape |
| `packages/codemode/` | Code-step package (study only; runs under provider host rules) |

Compare the lazy capability gateway (`search_capabilities`/`execute_capability`) to CAPABILITY-FABRIC. `/ee` is Enterprise-licensed — no reuse.

---

## Open Cowork — `opencoworkai/open-cowork`

| Pin | `e94ee2c50239e64ba0e18921e339d49b93dae31f` (v3.3.1, 2026-11-08) |
| App path | `src/main` (Electron main) + `src/renderer` |
| License | MIT |

### Read first

| Path (under pin) | Why |
|---|---|
| `src/main/agent/agent-runner-loop-guard.ts` | Non-progress / loop guard (maps to OpenHands non-progress proposal) |
| `src/main/schedule/scheduled-task-manager.ts`, `local-condition-checker.ts` | Scheduler + condition polling (TriggerHost / ON_CONDITION) — **roadmap: cron not yet shipped** |
| `src/main/sandbox/path-guard.ts`, `lima-bridge.ts`, `wsl-bridge.ts` | WSL2/Lima VM isolation + path-traversal/zip-slip guard (ENVIRONMENTS attestation) |
| `src/main/mcp/mcp-manager.ts`, `mcp-oauth.ts` | MCP transports + OAuth credential indirection |
| `src/renderer/components/` | ChatView / Sidebar / ContextPanel / TracePanel (TracePanel → Inspector only) |

ROADMAP Completed vs Planned: treat VM sandbox, PPTX/DOCX/PDF/XLSX skills, MCP connectors and GUI computer-use as shipped; scheduled tasks, first-class Linux builds and native tool schemas are Planned, not shipped.

---

## Rowboat — `rowboatlabs/rowboat` (NEW)

| Pin | `9052e28b9ede46478e04f9cf600c7ebaf5895dc7` (2026-11-08) |
| App path | `apps/x` (desktop), `apps/harbor` (agent runtime, spec-driven) |
| License | Apache-2.0 |

### Read first

| Path (under pin) | Why |
|---|---|
| `apps/harbor/SPEC.md`, `apps/harbor/CONTRACT.md` | Spec-driven agent-runtime contract (like LiteCowork SDD) |
| `apps/x/CODE_MODE_ENGINES_PLAN.md` | Code-mode engine plan |
| `apps/x/GRANOLA_PARITY.md`, `apps/x/MINI_APPS_PLAN.md` | Meeting-note parity + mini-apps surfaces |

Living knowledge-graph-of-your-work → Memory/Knowledge reference. Do not adopt harbor as Task authority.

---

## OpenLoomi — `melandlabs/openloomi` (NEW, same Tauri+Rust stack as LiteCowork)

| Pin | `2aca1016fa3065e2fee912afd0e94576d0483c66` (2026-11-08) |
| App path | `apps/web` (React) packaged as Tauri; `apps/web/src-tauri` (Rust) |
| License | Apache-2.0 |

### Read first

| Path (under pin) | Why |
|---|---|
| `apps/web/src-tauri/src/lifecycle.rs`, `close_behavior.rs`, `launch_mode.rs` | Tauri app lifecycle / quit-close / launch mode — direct Operator-vs-daemon reference |
| `apps/web/src-tauri/src/js_scheduler.rs` | In-process scheduler |
| `apps/web/src-tauri/capabilities/`, `permissions/` | Tauri capability/permission manifest discipline |
| `apps/web/src-tauri/src/audio_capture.rs` | Screen/audio understanding — **consent-gated RESEARCH only, never default** |

Attention/triage "surface only what needs you" parallels LiteCowork **Needs You**.

---

## OpenHands — `OpenHands/OpenHands`

| Pin | `22e3057c89b2e5245a5a36fd9c48e52c8a984ba4` (2026-11-06) |
| Engine pin | `OpenHands/software-agent-sdk` `f19fb2cf6a7f38266c6be954e91a9ef2a360bf9f` (2026-11-06) — agent loop, runtime, tools, agent-server |
| App path | `src/` (React renderer) + `electron/` (Desktop Host shell); the engine repo has `openhands-sdk/`, `openhands-agent-server/`, `openhands-tools/`, `openhands-workspace/` |
| License | MIT (both repos) |

**The `OpenHands` repo alone is the client shell.** Every runtime/loop/receipt path below except the shell rows lives in the engine pin; the desktop repo is a pure HTTP/WebSocket client of an agent-server (workspace-mode selector offers Docker "only when the server can run it": `src/utils/workspace-mode.ts` L10–L29). Third clone `OpenHands/typescript-client` (`5072d2a9`) is superseded — its code moved into the engine repo.

### Read first (architecture)

| Path (under pin) | Why |
|---|---|
| *(shell)* `electron/main.mjs`; `electron/preload-main.cjs` | Desktop Host shell: `nodeIntegration:false` + `contextIsolation:true` (L386–L403), popup/navigation policy (L427–L460); contextBridge-only surface `desktopShell`/`marsBridge` (preload L16–L47) |
| *(shell)* `src/utils/conversation-archive-status.ts`; `src/hooks/query/conversation-history-query-options.ts` | Runtime-unavailable presentation: `MISSING` ≠ `ERROR`, read-only chat/history only (L23–L50); history query keyed by host + session key so a re-provisioned runtime re-fetches (L70–L95) |
| *(engine)* `openhands-sdk/openhands/sdk/conversation/impl/local_conversation.py` | Run loop state machine, max-iteration/budget → `ERROR` + typed event (L2104–L2233), exception → error unless already surfaced (L2249–L2275), interrupt → `PAUSED` + `InterruptEvent` + synthetic error observations for orphaned actions (L2718–L2753, L2876–L2904), `rerun_actions` replay hazard warning (L3208–L3242) |
| *(engine)* `openhands-sdk/openhands/sdk/agent/agent.py` | Step: pending-action (implicit confirmation) execution first, condensation emitted as event, typed LLM-error recovery (L740–L930); tool call → `ActionEvent` + `AgentErrorEvent` on unknown tool/schema failure (L1311–L1375, L1415–L1508); execution asserts an `Observation` and pairs `action_id`/`tool_call_id` (L1546–L1608) |
| *(engine)* `openhands-sdk/openhands/sdk/conversation/state.py`; `openhands-agent-server/openhands/agent_server/models.py` | Persistence split: catalog (`meta.json`) vs `ConversationState` in `base_state.json` (+ event log) (models L104–L114, state L82–L250); runtime projection `ConversationRuntimeStatus{AVAILABLE,STARTING,MISSING,OWNERSHIP_LOST,ERROR}` + `can_resume` (models L50–L72); parent/child conversation link fields (models L274–L288) |
| *(engine)* `openhands-agent-server/openhands/agent_server/docker_runtime/registry.py` | Per-conversation container runtime: `--user <uid>`, `--cap-drop ALL`, `no-new-privileges`, `--ulimit core=0`, loopback-only port (L720–L760); lease-generation compare-and-stop `stop_if_idle` (L434–L479); attached-session eviction suppression (L259–L294); idle eviction of terminal conversations (L548–L615); health gate + startup rollback (L752–L793); owner-label orphan reaping (L296–L318) |
| *(engine)* `openhands-agent-server/openhands/agent_server/bash_service.py`; `.../models.py` | Command receipts: `BashOutput{command_id, order, exit_code=None⇒running, stdout, stderr}` + `BashError` (models L649–L692); persist-before-publish, chunked stdout/stderr with monotonic `order` (bash L346–L388); timeout = group SIGTERM→SIGKILL→`exit_code=-1` (L417–L436); error path always emits a terminal receipt (L462–L471); retention-managed file log (L507–L575) |
| *(engine)* `openhands-sdk/openhands/sdk/workspace/remote/remote_workspace_mixin.py` | Consumer-side receipt integrity: duplicate-event → hard `RuntimeError`, `exit_code=None` → `-1` + "Command timed out after N seconds", transport error → `exit_code=-1` never 0 (L196–L284) |
| *(engine)* `openhands-tools/openhands/tools/terminal/definition.py`; `.../terminal/terminal/terminal_session.py` | Terminal observation receipt: `exit_code=-1` = soft-timeout-incomplete, `timeout` flag, `CmdOutputMetadata`, `[Command finished with exit code N]` in model-visible text, truncation + file save (definition L146–L199); status machine `COMPLETED/NO_CHANGE_TIMEOUT/HARD_TIMEOUT` (session L39–L46) |
| *(engine)* `openhands-tools/openhands/tools/task/{definition,manager}.py`; `.../delegate/impl.py`; `openhands-sdk/openhands/sdk/subagent/scope.py` | Sub-agent delegation: `TaskStatus{RUNNING,COMPLETED,ERROR}` with resume-by-task-id reloading the same conversation id (manager L56–L66, L210–L253); stop/stuck/paused surfaced as ERROR with partial output (L420–L461); blocking parallel children over the **shared parent workspace** (delegate L287–L417); capability-subset scope `missing_from` + scope propagated to nested delegation tools (scope L46–L119); sub-agents always inherit exactly the parent's MCP servers, own declared servers dropped (L75–L95) |
| *(engine)* `openhands-tools/openhands/tools/child_conversation/{definition,impl}.py` | Host-mediated child conversation: synchronous POST `/api/conversations/{id}/children` with the server's session key, 300 s timeout, real observation `{conversation_id, parent_conversation_id, status, url, workspace, isolation}` (impl L36–L40, L118–L155); timeout message admits "The child may still have been created" (L128–L131); `isolation: worktree|shared` with honest branch semantics and "backends … may ignore this and report what was applied" (definition L38–L79) |
| *(engine)* `openhands-workspace/openhands/workspace/remote_api/workspace.py`; `.../cloud/workspace.py`; `.../agent_sandbox/workspace.py` | Remote-runtime contract: attach-or-create keyed by stable `session_id` (GET `/sessions/{id}`, `running|paused` attachable, paused→resume), then host/api_key swap and client reset (L141–L155, L157–L182); readiness classification transient vs **terminal** with pod logs/OOM hint, 300 s budget (L292–L367); pause/resume endpoints (L245–L281); close policy `keep_alive`/`pause_on_close` (L389–L416); Cloud sandbox states incl. auto-resume on `PAUSED` and terminal `ERROR|MISSING`, agent-server URL discovered from `exposed_urls` (cloud L303–L421), `pause()` raises `NotImplementedError` rather than faking support (L461–L473); suspend/resume re-patches `operatingMode` and **reconnects on a fresh port-forward with a rebuilt client** (sandbox L358–L387); provisioning payload pins `run_as_user/fs_group=10001` and `runtime_class` (API remote L198–L212) |
| *(engine)* `openhands-sdk/openhands/sdk/mcp/{config,tool,utils}.py`; `openhands-sdk/openhands/sdk/workspace/remote/base.py` | MCP as governed tool source: transports `stdio|http|streamable-http|sse` + credential union + per-server `timeout` (config L494–L549); failures/timeouts → `is_error=True` observations (tool L123–L246); `notifications/tools/list_changed` reconciliation and server-scoped names (utils L265–L370); secrets reach the agent as `LookupSecret` refs resolved lazily by the server, raw values never transit the SDK client (base L479–L535) |

### Copy / prototype / reject (proposal for owner review — no LiteCowork change is decided)

Evidence column = upstream `file:line` at the pins above. LiteCowork column names the owner document + section that *would* have to change.

| Mechanism | Verdict | Upstream evidence | LiteCowork owner (proposal) |
|---|---|---|---|
| Ordered, durable command receipt (persist-before-publish, `order` cursor, `exit_code=None`⇒running, separate service-error type) | copy | `bash_service.py` L346–L388; `models.py` L669–L692 | `docs/V1-INTEGRATED-TERMINAL-LLD.md` §"PTY and stream protocol" (L73) + §"Proposed functions" (L103) |
| Timeout as an explicit terminal receipt (`-1` + reason text) and truncation stated in the model-visible text | copy | `bash_service.py` L417–L436; `terminal/definition.py` L146–L199 | `docs/V1-INTEGRATED-TERMINAL-LLD.md` §"PTY and stream protocol" (L73); `docs/STATE-MACHINES.md` §"Effect and Evidence" (L885) |
| Consumer-side receipt integrity: duplicate detection, never assume exit 0, transport error ≠ success | copy | `remote_workspace_mixin.py` L218–L284 | `docs/ARTIFACTS-EVIDENCE.md` §"Effect service" (L157) + §"Evidence" (L210) |
| Runtime availability projection (`MISSING` vs `ERROR` vs `OWNERSHIP_LOST`, `can_resume`) with a fixed read-only UX for runtime-backed affordances | copy | `models.py` L50–L72, L381–L387; shell `conversation-archive-status.ts` L5–L50 | `docs/ENVIRONMENTS.md` §"Attempt binding" (L373) + §"Provider result and isolation rules" (L346); `docs/STATE-MACHINES.md` §"Environment" (L596) |
| History readable while the runtime is absent; UI query keyed by host/session identity so a re-materialized runtime re-fetches | copy | `event_service.py` L540–L551; shell `conversation-history-query-options.ts` L84–L95 | `docs/EVENTS.md` §"Ephemeral streams" (L876); `docs/AGENT-FABRIC.md` §"Session lifetime and conversation continuity" (L376) |
| Stream reattach contract: resend mode + cursor (`resend_mode=all|since`, `after_timestamp`), history-then-subscribe-from-cursor | copy | `sockets.py` L229–L347 | `docs/EVENTS.md` §"Ephemeral streams" (L876); `docs/AGENT-FABRIC.md` §"Normalized events" (L432) |
| Idle eviction with lease-generation compare-and-stop + "an attached viewer suppresses reclamation" | prototype | `registry.py` L259–L294, L434–L479, L548–L615 | `docs/ENVIRONMENTS.md` §"Lifecycle" (L252) + §"Lifetime and reuse" (L262); `docs/STATE-MACHINES.md` §"EnvironmentControlLease" (L651) |
| Health-gated readiness + startup rollback + owner-labeled orphan reaping on boot | copy | `registry.py` L296–L318, L752–L793 | `docs/RUNTIME-LIFECYCLE.md` §"Lazy capabilities, Environments, and applications" (L490); `docs/ENVIRONMENTS.md` §"Provider result and isolation rules" (L346) |
| Attach-or-create keyed by a stable session id; readiness classification distinguishes transient from terminal failure and reports evidence (pod logs/OOM) | prototype | `remote_api/workspace.py` L141–L233, L292–L367 | `docs/RUNTIME-MESH.md` §"Runtime identity" (L41) + §"Pairing" (L82); `docs/ENVIRONMENTS.md` §"Provider contract" (L9) |
| Suspend/resume re-materializes a **new** endpoint: reconnect and rebuild the client, never reuse the stale address | prototype | `agent_sandbox/workspace.py` L358–L387 | `docs/RUNTIME-MESH.md` §"Unexpected runtime loss" (L677); `docs/ENVIRONMENTS.md` §"Checkpoint semantics" (L322) |
| Provider capability honesty: an unsupported operation raises/marks unsupported instead of simulating | copy | `cloud/workspace.py` L461–L473 | `docs/ENVIRONMENTS.md` §"Consumer file-root mount modes" last para (L410) — already the stated rule; extend to provider ops |
| Host-mediated child conversation with a real observation, declared isolation and an ambiguity-honest timeout | copy | `child_conversation/impl.py` L36–L40, L118–L155; `definition.py` L38–L79 | `docs/DELEGATION.md` §"Composer conversation versus Task worker dispatch" (L288); `docs/ENVIRONMENTS.md` §"GitWorktree" (L228) + §"Environment sharing and write coordination" is DELEGATION L769 |
| Sub-agent capability subset (child may not use tools the parent lacks; scope propagates to nested delegation tools) and mandatory MCP inheritance (parent's servers only, never merged) | prototype | `subagent/scope.py` L46–L119, L75–L95; `task/manager.py` L389–L407 | `docs/DELEGATION.md` §"Native harness integrity" (L37) + §"Native child model behavior and observability" (L315) |
| Sub-agent budget/iteration inheritance and metrics replace-not-merge across resume | copy | `task/manager.py` L262–L278, L500–L508; `delegate/impl.py` L373–L382 | `docs/DELEGATION.md` §"Cost, budgets, and performance projections" (L677) |
| MCP as a governed tool source: transport+credential union, per-server timeout, failure→observation, tool-list reconciliation, host-side credential indirection | prototype | `mcp/config.py` L494–L549; `mcp/tool.py` L123–L246; `mcp/utils.py` L265–L370; `workspace/remote/base.py` L479–L535 | `docs/AGENT-FABRIC.md` §"AgentCapabilities" (L47); `docs/AGENT-CONTROL.md` §"5.1 AgentRegistryEntry" (L272) + §"6.2 Non-secret configuration" (L401); `docs/STATE-MACHINES.md` §"CapabilityGrant, SecretLease, Activation, and Invocation" (L669) |
| Non-progress guard: named thresholds (repeating action/observation, action/error, monologue, alternating), one nudge per streak, terminal `STUCK` state | prototype | `stuck_detector.py` L54–L146; `local_conversation.py` L808–L830 | `docs/AGENT-CONTROL.md` §"15. Failure and recovery" (L1143); `docs/STATE-MACHINES.md` §"AgentSession" (L236); `docs/FAILURE-RECOVERY.md` §"Failure matrix" (L42) |
| Acknowledgment-only client tools (`"Tool call dispatched to client."`, real outcome only via a later prefixed `user` message) | reject | `client_tool.py` L1–L10, L160–L172; shell `src/services/client-tool-result.ts` L5–L30 | would contradict `crates/domain-effects/src/lib.rs` L606–L611 (completion evidence from the authoritative store, never agent output) |
| Client-side idempotency ledger for non-idempotent UI-triggered launches | copy (only if a UI-executed action ships) | shell `child-conversation-launch.ts` L506–L535 | `docs/STATE-MACHINES.md` §"RichPresentation" (L1377) + §"CapabilityGrant…Invocation" (L669) |
| Container-default isolation treated as the whole sandbox story (no seccomp/AppArmor/userns/read-only rootfs/egress policy anywhere in either repo) | reject | `registry.py` L720–L748 (complete flag set); repo-wide grep for `seccomp|apparmor|userns|cap_add|read_only|network=` finds none in this path | `docs/ENVIRONMENTS.md` §"Isolation" (L101) + §"Provider result and isolation rules" (L346–L371) already demand more |
| `declared_resources → declared=True` on a tool that spawns other agents (declares "no resources" while children write the same workspace) | reject | `task/definition.py` L185–L186, L199–L205 | authority is the lease/fence: `docs/ENVIRONMENTS.md` §"Attempt binding" (L385–L391) |
| Control/result messages posted into the transcript as real `user` messages, then hidden by the renderer | reject | shell `src/services/client-tool-result.ts` L5–L14; `.../should-render-event.ts` L63 | use session/normalized events: `docs/EVENTS.md` §"Ephemeral streams" (L876); `docs/AGENT-FABRIC.md` §"Normalized events" (L432) |
| In-process sub-agents over the shared parent workspace, blocking parent, failure taxonomy `ERROR`-only (stop/stuck/paused collapsed into an error string) | reject | `delegate/impl.py` L287–L417; `task/manager.py` L56–L66, L420–L461 | `docs/DELEGATION.md` §"LeadFailoverPolicy" (L536) + §"Current source boundary" (L881); `docs/TASK-RUNTIME.md` §"Cancellation" (L480) |
| Replay recorded actions as a recovery strategy | reject | `local_conversation.py` L3208–L3242 | `docs/FAILURE-RECOVERY.md` §"Reconcile first" (L101) |

### Not verified (do not treat as findings)

- `openhands-agent-server/.../conversation_service.py`, `docker_runtime/mediation.py`, `sdk/subagent/registry.py` read only in outline; runtime claims rest on `registry.py` + `event_service.py` + `models.py`.
- Whether `delete_on_close=True` (task/manager.py L234, L339) deletes persisted task files in addition to dropping in-memory handles — the module docstring claims task conversations are persisted for resume; I did not reconcile the two.
- Nested-container support on the **local** Docker path (`sysbox-runc` default applies to the remote-runtime path, `remote_api/workspace.py` L61–L63); the local path's DinD story is unverified.
- Electron hardening beyond the window options/popup handlers/preload surface read above (886-line main process, not read end to end).
- LiteCowork-side code was read to confirm *absence* or *presence of text* only; no LiteCowork behavior was executed.

---

## Procedure for new rival repo

1. Pin SHA; record in [REFERENCE-IMPLEMENTATIONS.md](REFERENCE-IMPLEMENTATIONS.md).  
2. List **entry points**: main, preload/bridge, agent loop, storage, scheduler.  
3. Add **3–10 files** to this index with *why*.  
4. Document **copy / prototype / reject** in feature `RESEARCH.md`.  
5. Add promoted USER journeys to [COMPETITOR-PROMOTED-USE-CASES.md](COMPETITOR-PROMOTED-USE-CASES.md).  
6. Update [VIABILITY-VS-COMPETITORS.md](VIABILITY-VS-COMPETITORS.md).

---

## LiteCowork reference desktop audit status

ZCode + DSH: **targeted component audit complete** in DESKTOP-REFERENCE-IMPLEMENTATION-AUDIT (2026-10-31); pins refreshed 2026-11-08 (ZCode 3.17.2, DSH 0.2.2-rc.1). Re-read Host/IPC when deepening against the new pins.

AionUi + OpenDot: **2026-11-01** shallow architecture pass; deepen when SDD 008/009/011 enters IN_REVIEW.

OpenWork + Open Cowork + Rowboat + OpenLoomi: **2026-11-08** shallow architecture pass (read-first indexes above). OpenLoomi is the closest same-stack (Tauri+Rust) reference; deepen its lifecycle/scheduler modules when SDD 010/011 enters IN_REVIEW.
