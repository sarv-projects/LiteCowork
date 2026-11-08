# Research - Desktop Shell IPC Settings and Runtime Health

**Feature:** 011 | **Checked:** 2026-11-06 (source crosswalk and NeuralAgent screenshot comparison).

## Sources and authorities

Authorities: docs/RUNTIME-LIFECYCLE.md, docs/LOCAL-OPERATOR-IPC.md, docs/DESKTOP-SHELL-LLD.md, docs/V1-SETTINGS-CONTRACT-LEDGER.md, docs/V1-UI-SCREEN-CONTRACT.md, docs/SEMANTIC-SERVED-UI-MAP.md, docs/SETTINGS-AND-SHELL-SURFACE-AUDIT.md, docs/V1-TELEMETRY-USAGE-RESILIENCE-LLD.md, docs/V1-AGENT-DISCOVERY-AND-CATALOG-LLD.md. The Settings control ledger is normative target, not evidence of all controls being served.

## Borrowable-design sources (pinned)

| Source | Exact pinned URL | License | What informs this feature | Verdict |
|---|---|---|---|---|
| ZCode thin-shell / Host / CLI ownership split (`zai-org/ZCode`) | https://github.com/zai-org/ZCode/tree/a9fe401eede6e6e5e01be9048679c206fe6f02d9 | Apache-2.0 + NOTICE | `packages/desktop` Main/Renderer/Host/CLI ownership split and plugin guest sandbox inform keeping the Tauri shell (untrusted renderer) strictly separate from the Runtime-local Host, so the shell is never the security boundary and native handles/credentials never cross into the WebView. | PORT the ownership-split pattern only; do not drop in React or the ZCode Host runtime. Read-first paths in REFERENCE-REPOS-CORE-ARCHITECTURE-INDEX §ZCode. |
| DeepSeek Harness typed IPC with sender assert (`deepseek-ai/deepseek-harness`) | https://github.com/deepseek-ai/deepseek-harness/tree/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159 | MIT at checked root | `apps/desktop` typed IPC with sender assertion, contextBridge preloads and host lifecycle/deadlines inform the authenticated local IPC boundary (AC-001/AC-002), the no-bearer-fallback rule and the quit-with-active-work UNKNOWN UX for drain (AC-012). | PROTOTYPE the typed-IPC/sender-assert and quit-drain patterns against LOCAL-OPERATOR-IPC and RUNTIME-LIFECYCLE; do not vendor the Electron host. |
| OpenLoomi Tauri capability/permission manifests (`melandlabs/openloomi`) | https://github.com/melandlabs/openloomi/tree/2aca1016fa3065e2fee912afd0e94576d0483c66 | Apache-2.0 at checked root | Same Tauri + Rust stack as LiteCowork; its `apps/web/src-tauri` Tauri capability/permission manifests and lifecycle/close-behavior/launch-mode modules are the closest same-stack reference for scoping shell-to-Host capabilities and honest Runtime close/drain behavior. | PROTOTYPE the Tauri capability/permission manifest scoping and close-behavior patterns; same-stack so they port cleanly. Flag screen-understanding/audio capture as out-of-scope consent-gated RESEARCH. |

No ZCode, DeepSeek Harness or OpenLoomi code is imported; all are read at the pinned SHAs. OpenBot (PolyForm Noncommercial) is not cited here and would be reference/UX only with no code reuse if it were.

## External comparison

The supplied NeuralAgent screenshots demonstrate: persistent Settings category navigation beside a focused settings panel; Installed and Available agent groupings; notification sound/delivery choices; keep-awake and restart-resume policies; an Agents work overview with Needs You / Working / Done summary filters; and a usage dashboard with time range, per-agent limits, token types and model cost breakdown.

The GitHub repository https://github.com/LePro10/NeuralAgent and current public product material are not a single immutable product version. The older source inspected for this comparison is a Tauri/Rust daemon + shared React UI + Android/relay architecture with daemon-owned terminal sessions, Neural orchestration and project Autopilot. Current public NeuralAgent feature pages describe a newer computer-use product. Adopt the UI hierarchy and selected observer/daemon ownership patterns, not the whole app architecture or its assumptions. External URL/page freshness and version must be pinned before any algorithm or code reuse.

## Decisions adopted for LiteCowork

- Settings is a category panel, not a second application navigator. Keep category rail + focused content panel.
- Agent catalogue and per-agent profile are distinct. Installed vs Available is explicit; source/installability is verified before an Install action appears.
- Detection, installation, configuration, authentication, descriptor freshness, binding and session readiness are independent state axes.
- Agent status cards and install controls live under Settings > Agents. The default model/auth selection stays native-harness-owned; no universal provider/model router.
- Needs You / Working / Done are summary filters over the existing Work/Task owner projections, not new permanent sidebar routes or another task engine.
- Notifications separate sound and OS delivery. A Try-it action is a local preview unless a real OS provider acknowledges delivery.
- Keep-awake is an OS power assertion. Resume-after-restart only covers LiteCowork-owned durable work whose recovery contract exists; it does not resurrect arbitrary native CLI processes.
- Usage is a first-class read-only projection backed by immutable inference leaves plus a separate quota snapshot ledger. Cost estimates, reported/billed money and subscription entitlements are separate.

## Current implementation evidence

The active App.tsx Settings route mounts SettingsView with four categories only: General, Workspaces, Agents and Runtime. Workspace selection/create/instruction revisions/archive, a subset of Agent catalogue/profile probe/binding operations and Runtime start/refresh/lifecycle reads connect to Tauri/Operator paths. General appearance selectors are component-local state that is neither persisted nor applied; shortcut labels are display-only. WorkspaceRoot management, policy mutation and Runtime enrollment are not rendered in the mounted component. The exported SettingsPage is unmounted. Previous nine-category and Browser/cache/TLS/Shell Engine/approval/Credential Store claims are stale; Workbench Browser/Terminal are separate Feature 008 surfaces. See SETTINGS-AND-SHELL-SURFACE-AUDIT.md.

The current AgentCatalogSettings supports bounded inventory observations, ACP registry metadata and Codex/OpenCode profile probing. There is no trusted AgentModule install/update/repair action or native sign-in experience. Control descriptor and credential routes return 503; native config/auth operations remain target-only. See Feature 002 and IMPLEMENTATION-STATUS.md.

The current route/source search found no complete historical usage projection/dashboard. Existing environment budget counters and worker cost observations are not the same as a per-inference accounting ledger. The telemetry LLD is not yet served through the complete OpenAPI/Operator/SQLite/Tauri/UI chain.

## Non-adoption and constraints

- No direct execution of curl/bash/package commands supplied by remote registry metadata.
- No synthetic Ready/Working/Delivered/Installed statuses.
- No model/provider abstraction that replaces native harness model/auth behavior.
- No second task list or duplicate Task count.
- No guessed tokens, API-equivalent cost presented as billed amount, flat-plan quota converted into dollars, fuzzy dedupe or cross-currency total.
- No process resume promise without exact owner/session recovery identity.

## Validation limitation

This is a source and document audit plus screenshot comparison. No Notification OS provider, keep-awake native integration, generic resume workflow, AgentModule install/auth route or Usage projection was executed as part of this documentation pass. Those remain implementation and OS qualification work.