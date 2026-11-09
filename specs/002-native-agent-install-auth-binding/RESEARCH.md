# Research - Native Agent Install, Auth, Binding and Agent Profile UX

**Feature:** 002 | **Checked:** 2026-11-06 (active source/UI audit and screenshot/competitor comparison; no native install or provider session executed). Re-traced 2026-11-09 for the OpenCode binding break (SPEC §4a).

**Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.

## Primary authority
Canonical design and state ownership: docs/AGENT-CONTROL.md, docs/AGENT-FABRIC.md, docs/V1-AGENT-DISCOVERY-AND-CATALOG-LLD.md, docs/V1-SETTINGS-CONTRACT-LEDGER.md, docs/V1-UI-SCREEN-CONTRACT.md and root ARCHITECTURE.md. Feature 002 owns native AgentModule/profile/install/auth/binding boundaries. LiteSPM owns MCP/skill/plugin/capability-package lifecycle. Feature 001 owns Conversation admission/session execution. Feature 011 owns Settings shell/control authority. The agent catalogue LLD is normative target, not current implementation proof.

## Borrowable-design sources (pinned)

| Source | Exact pinned URL | License | What informs install/probe/auth | Verdict |
|---|---|---|---|---|
| DeepSeek Harness `apps/desktop` typed IPC with sender assert | https://github.com/deepseek-ai/deepseek-harness/tree/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159 | MIT | Typed IPC with sender assertion, contextBridge preloads and host lifecycle/deadlines inform the authenticated Operator boundary for install/probe/auth commands and the quit-with-active-work UNKNOWN UX for an in-flight probe. | Prototype the quit-drain against RUNTIME-LIFECYCLE and port the typed-IPC/sender-assert pattern; do not vendor the Electron host. |
| ZCode `packages/desktop` thin-shell / Host / CLI split | https://github.com/zai-org/ZCode/tree/a9fe401eede6e6e5e01be9048679c206fe6f02d9 | Apache-2.0 + NOTICE | The Host/shell ownership split informs keeping the Settings catalogue UI (shell) separate from the Runtime-local lifecycle/probe Host, so native handles and credentials never cross into the WebView. | Port the ownership-split pattern only; do not drop in React or the ZCode Host runtime. |

No DeepSeek Harness or ZCode code is imported; both are prototype/reference inputs. OpenBot (PolyForm Noncommercial) is cited below for model/effort-picker UX only and is explicitly reference/UX with no code reuse.

## External comparison and adoption
NeuralAgent public material describes a desktop workflow around discovering/using agents, configured/installed state, persistent agent work views, model routing, local voice, scheduling and computer-use. Its historic public repository describes a different Electron/React + Python/FastAPI architecture and plugin-like agent roles; current public product pages now describe a later computer-use product. Do not treat those web pages and the old code repository as one versioned implementation or copy either architecture wholesale. NeuralAgent's Settings screenshots are useful information-hierarchy references: persistent category navigation, installed vs available agents, notification preference, advanced runtime behavior and a separate cost/usage surface. Its Agents overview uses three summary filters (Needs You, Working, Done) and a grouped work list rather than separate permanent sidebar pages. Those counts/statuses must be real owner projections in LiteCowork.

Adopt:
- agent catalogue and agent profile are separate levels;
- installed/detected/configured/authenticated/ready are independent axes;
- installed and available distributions are separate sections with source, version, compatibility and action;
- status groups such as Needs You / Working / Done are filters over durable Work/Task states, not independent navigation routes;
- notification preferences are separate from work state and reflect the real notification delivery/attention queue;
- advanced controls distinguish local preference from native process/runtime behavior and show restart/recovery semantics;
- Usage is a first-class read-only observability surface, drillable by actual agent/model/session/Task with confidence/coverage.

Reject:
- a single Connected toggle;
- treating a registry listing as a safe install command;
- a global model/provider selector that overrides an agent's native model/auth contract;
- a top-level Working list that duplicates Work/Task or Conversation state;
- usage totals inferred from screenshots, flat-plan token guesses or assumed API list price;
- visual statuses for process, resume, keep-awake or notifications without owner service/OS observation.

## Screenshot-specific UI interpretation
1. Settings Agents: default agent selection is native-harness-owned and applies only to new work; installed versions are listed with exact executable/version and external official link; available candidates show install action only when a trusted install plan exists.
2. Notifications: explicit choices combine sound and OS notification delivery (both, sound only, notification only, neither), with quiet/in-focus behavior; a 'Try it' control is a real local preview that does not send a notification to another device.
3. Advanced: keep-awake choices are OS power assertions with distinct battery/power semantics; resume-after-restart is a scheduler/recovery policy for admitted agent jobs, not a promise that arbitrary external native sessions can be restored.
4. Agents overview: 3 status summary tiles are filters, not left-sidebar routes. LiteCowork's primary sidebar remains Conversations, Coworker roster, Work, Needs You, Automations, Library, Discover and Settings; Agents work should be accessible as a Work filter/overview, not a duplicate global task engine.
5. Usage: dashboard quick ranges Today, 7 days, 1 month, 3 months, 1 year and All time, plus YTD/Custom in expanded filters. Grouping and filters are governed by telemetry LLD and existing ledgers.
6. Work panels: keep separate Conversations and native Terminal panes within a Task/work context only when actual session/process identity exists; do not promote mock terminal output to durable completed work.

## Current LiteCowork source/UI evidence
- `AgentCatalogSettings` currently lists 15 curated candidates and ACP registry entries; registry results are read-only and map to NO_ADAPTER.
- Local inventory probes version/path metadata for a bounded candidate set; auth UNKNOWN, readiness NOT_PROBED. No actual AgentModule install/update action exists.
- Profile probe supports Codex/OpenCode only, Runtime enrollment required. Codex App Server initialize/account/model-list is observed; OpenCode is recorded incompatible. Neither guarantees an end-to-end Conversation session.
- Descriptor read/refresh and credential submission return 503 DEPENDENCY_UNAVAILABLE; auth-begin and native-config-open are target-only.
- Binding list/create/enable/disable/default actions are served; enable currently lacks required fresh AgentControlDescriptor gate.
- Conversation supervisor dispatches OpenCode only, but current OpenCode profile probe is incompatible; Codex can be bound but is not dispatched by the active supervisor.
- UI currently has no Install/Update or native sign-in flow; usage detail/projection is not served.

## V1 boundary
Deliver one complete exact native agent journey first. Other agents may be listed as detected/available/unsupported but must not be presented as runnable until their install/auth/descriptor/session contract is independently qualified on a declared OS/Runtime. Do not use catalog breadth as a V1 acceptance metric.


## OpenBot model/effort picker comparison — 2026-11-06

Pinned source: nightly-labs/openbot@6759fa3ee857f4f4966865e12de99ae84ab6de07 (package 0.62.0). The shared AgentSettingsPanel and ProviderModelPicker show a compact, searchable model list attached to one selected agent/profile/account, with a selected marker and an optional effort row. Provider runtime adapters supply model catalog and account-specific details; this is not a global model router.

Adopt the compact interaction hierarchy only: agent/profile identity, searchable exact-provider model list, selected/default marker, and effort only when the exact descriptor supports it. Keep provider-specific modes outside the generic model list. The model control is distinct from the Task agent/Coworker selector. Do not mix model options from different providers/accounts.

The OpenBot package declares PolyForm Noncommercial 1.0.0 and its README describes a development preview with danger-full-access / approvalPolicy never. No code or permission model is reused. Full source pin and limitations: [OPENBOT-REFERENCE-IMPLEMENTATION-STUDY.md](../../docs/OPENBOT-REFERENCE-IMPLEMENTATION-STUDY.md).

LiteCowork's model/effort picker remains target-only until fresh AgentControlDescriptor read/refresh, adapter compatibility, account scope, and an exact native Conversation/Task send path are all qualified. Existing discovery/profile/binding evidence does not establish that a selected model is dispatchable.


## Claude Code CLI as the second module (B-11, accepted 2026-11-09)

The inventory already lists a `CLAUDE_CODE` candidate (`agent_id: "claude-code"`, executable `claude`) in `apps/litecoworkd/src/agents/native_cli.rs`. That is version/path detection only. There is no Claude Code profile probe, descriptor, auth observation or supervisor dispatch arm. 002-T12 pins the exact CLI version, the non-interactive streaming transport and the native auth method in a conformance fixture by running the installed CLI and reading its own help/version output on each release-matrix OS; it does not infer flags from third-party listings. The dispatch arm itself is Feature 004 (B-16).

## Implementation efficiency and language-placement observations (2026-11-09)

These are source-grounded observations recorded for the implementer. Each is folded into the task named; none changes a locked decision.

- **Stringly-typed provider dispatch (Rust).** `execute_agent_turn` matches `provider_key` string literals, which is how the `OPENCODE`/`opencode` mismatch went unnoticed. A closed `NativeProviderKey` enum parsed once removes the class of bug (002-T01).
- **Duplicate executable resolution (Rust).** `resolve_local_executable` in `operator.rs` and `find_opencode_executable` in `agents/supervisor.rs` implement different search rules for the same binary. One resolver in `apps/litecoworkd/src/agents/` plus dispatch from the probe-registered `endpoint_ref` gives one identity and avoids a second PATH walk per turn (002-T01).
- **Profile lookup by full scan (Rust).** The supervisor calls `list_agent_profiles` and then searches linearly for one `agent_profile_id` on every turn. A keyed `get_agent_profile` read is cheaper and narrower (002-T01, shared file with Feature 004).
- **Descriptor rules duplicated across Rust and TypeScript.** `apps/litecoworkd/src/agents/control_descriptor.rs` validates descriptors in Rust, while `apps/litecowork-ui/src/conversations/conversation-agent-surface.ts` re-derives readiness in TypeScript and has no production caller. The daemon should own the derivation and serve a projection; the TypeScript side should only parse and render, so freshness rules have one implementation (002-T05, 002-T09).
- **Generated constant types.** `operator-api.ts` encodes `profile_admission_supported: false` as a literal type. After the OpenAPI change the file must be regenerated with `pnpm --dir apps/litecowork-ui run api:generate` and checked with `api:check`, not hand-edited (002-T01).
- **Large single files.** `apps/litecowork-ui/src-tauri/src/lib.rs` (about 6,100 lines) holds the agent profile/binding Tauri commands inline (`probe_agent_profile`, `create_agent_binding`, `enable_agent_binding`), unlike the per-domain `*_bridge.rs` modules. `apps/litecoworkd/src/operator.rs` (about 9,500 lines) holds the agent probe handlers. New 002 commands go into `agent_control_bridge.rs` or a new `agent_binding_bridge.rs` rather than growing `lib.rs`; moving existing commands is optional and only with no behavior change (002-T05, 002-T10).
- **Per-probe server spawn.** Each OpenCode probe spawns and stops a server (8 s startup budget). That is correct for a bounded read-only probe; the B-12 probe budget (3 per minute) bounds the cost. No change proposed.

## ACP client SDK pin (P-11, owner approval 2026-11-09)

Task 002-T17 defines the native-agent adapter as an Agent Client Protocol (ACP) client (register A-54). The ACP SDK version and the exact protocol revision must be pinned here from the published ACP SDK at implementation time, not inferred from CLI output or a marketplace listing. Record: the SDK package name and exact version, the ACP protocol revision it implements, the per-provider launch catalog entries (command, args, working-directory policy) for each enrolled provider, and the mapping from ACP `session/update` kinds to the internal normalizer events shared with Feature 001. The borrowable mapping and rationale are in [../../docs/REFERENCE-SYNTHESIS-AND-V1-FEATURE-ALIGNMENT-2026-11.md](../../docs/REFERENCE-SYNTHESIS-AND-V1-FEATURE-ALIGNMENT-2026-11.md) section 1 and register row A-54 in [../../docs/CONTRADICTION-AND-ALIGNMENT-REGISTER.md](../../docs/CONTRADICTION-AND-ALIGNMENT-REGISTER.md); they are not restated here. Pin status until the published SDK is fetched on the implementation host: TARGET-pending-published-ACP-SDK.

### ACP client spine implementation note (task 001-T04/002-T17, 2026-11)

The ACP client spine is implemented as native LiteCowork Rust in
`apps/litecoworkd/src/agents/adapter.rs` (no external paseo code imported).
Pinned values used by that spine:

- ACP protocol revision: `1` (constant `ACP_PROTOCOL_VERSION`), matching the
  `ACP_PROTOCOL_VERSION = 1` of the two independent production ACP clients
  surveyed (paseo and open-design) in
  `../../docs/OPEN-DESIGN-CAPABILITY-DEEP-DIVE-2026-11.md`.
- Method names: `initialize`, `session/new`, `session/prompt`,
  `session/cancel` (client to agent); `session/update`,
  `session/request_permission`, `fs/read_text_file`, `fs/write_text_file`,
  `terminal/create`, `terminal/output`, `terminal/kill`, `terminal/release`
  (agent to client, owned by the LiteCowork Client).
- Per-provider launch catalog (`acp_launch_catalog`): Claude Code
  (`claude-code-acp --acp`) and Codex (`codex acp`), each selected by exact
  provider identity, never a brand substring. OpenCode v2 HTTP is deliberately
  NOT an ACP launch target: it stays the one non-ACP adapter
  (`OpenCodeAdapter`) until its own ACP route qualifies.
- `session/update` kind mapping: `agent_message_chunk` -> answer text,
  `agent_thought_chunk` -> reasoning (kept separate), `tool_call` /
  `tool_call_update` -> tool activity, `plan` -> todo/plan. Correlation is on
  the `sessionId` in the update envelope; a foreign session update cannot
  mutate another session's state.

No published external ACP SDK crate was fetched or vendored in this sandbox
(no external registry access); the pin above is the protocol revision and
method surface the native spine speaks. When a published ACP SDK is adopted on
the implementation host, replace the `TARGET-pending-published-ACP-SDK` status
with the exact SDK package name and version and reconcile it against this
protocol revision.

