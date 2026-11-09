# UI behavior - Desktop Shell IPC Settings and Runtime Health

**Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.

Behavior, not pixels. The screen inventory is [V1-UI-SCREEN-CONTRACT.md](../../../docs/V1-UI-SCREEN-CONTRACT.md). Per-control authority is [V1-SETTINGS-CONTRACT-LEDGER.md](../../../docs/V1-SETTINGS-CONTRACT-LEDGER.md); current evidence is [SETTINGS-AND-SHELL-SURFACE-AUDIT.md](../../../docs/SETTINGS-AND-SHELL-SURFACE-AUDIT.md) and [IMPLEMENTATION-STATUS.md](../../../docs/IMPLEMENTATION-STATUS.md). This contract defines the Settings host and common state behavior; domain tabs retain their canonical owner contracts.

## Common Settings shell

Persistent category navigation with search at left and one focused content panel at right. Desktop keeps both visible; narrow windows use a keyboard/focus-managed category drawer. Category switching preserves scope and search. Every control displays scope and source class: local UI preference, native OS action/observation, Operator-committed domain state, read-only observation, pending/ambiguous or unavailable.

## Category inventory

General; Notifications; Workspaces; Agents; Connections & Tools; Local Models; Coworkers & Subagents; Runtime; Terminal; Browser & Computer; Memory & Knowledge; Security & Privacy; Storage & Backup; Usage & Monitoring; Advanced; About & Diagnostics. A category is shown only when it has useful content; target-only controls remain hidden or non-interactive with a concrete prerequisite. The full operation inventory and per-action failure semantics live in the Settings ledger.

## Specific panel rules

- **Notifications:** four choices (sound + OS notification, sound only, notification only, neither); quiet hours/classes/timezone; Try it previews sound locally and must not claim OS delivery without provider acknowledgement. Notification queue differs from Needs You.
- **Advanced:** keep-awake Off / while agents-work (battery included) / while agents-work on power only; each requires native OS power assertion. Resume-after-restart covers only admitted LiteCowork-owned jobs whose recovery contract is qualified; not arbitrary native CLI sessions. Runtime startup mode is independent.
- **Agents:** catalog plus profile sections; independent detection/install/config/auth/readiness; install preview then confirm; sign in via exact native method; descriptor-only capability/options; Workspace binding and defaults affect future admissions only. Feature 002 owns details.
- **Usage:** quick ranges Today / 7 days / 1 month / 3 months / 1 year / All time, with YTD/Custom advanced filters. Agent, harness, binding, model, provider, Coworker, session, Task, Attempt, lead/subagent, Runtime, source/confidence and outcome filters. Leaf drill-down shows provenance, not prompt/response. Estimates, billed amount and quota are separate. The projection is target-only until the telemetry owner chain is served.

## Commands and action contract

The UI renders owner state and sends only the command named by the canonical API. It does not encode a second domain state machine. Every action has a defined enabled predicate, scope, expected version/idempotency where required, loading/pending, acknowledged success, denial/failure/unknown result and recovery. LOCAL_PREF commits locally. Native actions require a verified OS result. Operator mutations require committed receipt/version. Read-only observations display source and age. TARGET/unserved controls send no mutation. Preserve draft and last committed value on failure.

## Current source boundary — 2026-11-06

The active Settings route mounts `SettingsView` with four categories only (General, Workspaces, Agents, Runtime). The 16-category inventory above is target-only. The mounted component has no category search or category drawer. Its appearance selectors are component-local state, not applied or persisted preferences; the displayed keyboard shortcuts are not wired to matching handlers. WorkspaceRoot controls, Workspace policy mutation and Runtime enrollment are not rendered. The exported `SettingsPage` in App.tsx is unmounted. Do not count its controls, passed-but-unused props, Tauri commands or target ledger entries as visible Settings controls. The mounted view also omits Runtime error/loading display, Workspace-create pending-journal retry affordance and Workspace-instruction conflict recovery. App bootstrap attempts Runtime auto-start when the daemon process is absent and the executable is available. See `docs/SETTINGS-AND-SHELL-SURFACE-AUDIT.md` for exact source mapping.

## Validation

Cover empty, stale, offline, denied, stale version, unsupported OS, unqualified provider, ambiguous timeout, cancellation, restart and recovery. Keyboard navigation, focus restore, screen reader labels, high contrast and reduced motion are required. Semantic Conversation output remains available when a rich renderer fails.

## Revision 1.0 accepted control states (2026-11-09)

| Control | Location | Enabled when | Pending / success / failure / unavailable |
|---|---|---|---|
| Workspace rename | Settings > Workspaces | Workspace ACTIVE, Operator ready, rename operation served | Saving; committed name from receipt; conflict shows latest; archived shows read-only |
| Archive Workspace | Settings > Workspaces (exists) | Workspace ACTIVE | Two-step confirm; "Archiving"; read-only after receipt |
| Preview stop / Stop safely | Settings > Runtime and Advanced | Stop route served and preview verdict QUIESCENT | Preview lists running/unknown items; Stop disabled with reason otherwise; never "Stopped" before STOPPED |
| Keep-awake | Settings > Advanced | OS provider present on this row | HELD only after OS confirms; UNSUPPORTED with reason; released on drain |
| Theme, density, text scale, reduced motion | Settings > General | Local preference store wired and applied | Applied immediately and persisted; otherwise removed (not shown inert) |
| Shortcut list | Settings > General | Generated from the registered shortcut registry | Only registered shortcuts are listed |
| Agents screen | Settings > Agents | Per Feature 002 served operation | Descriptor/credential operations that return 503 render unavailable with reason |
| Notifications | Settings > Notifications entry | Always (link) | Links to Feature 010 quiet-hours contract; no local quiet-hours control |
| Transport status | Settings > Runtime | Always | "Local IPC (named pipe)" on Windows, "Local IPC (socket)" on Linux; never "Localhost" |

Keyboard baseline (app-wide): primary navigation, New Chat, Conversation composer, Workbench and Settings are reachable with Tab/Shift+Tab; drawers and dialogs trap focus and return it on close; Escape closes the topmost drawer or dialog; focus is always visible.

## Cross-layer traces (011-T16)

Symbols below were checked against the source on 2026-11-10. "Unverified Tauri" means the command exists in `apps/litecowork-ui/src-tauri/src` but `src-tauri` cannot be compiled in the Linux container (gdk-3.0 missing), so only its pure decoder or its source text was reviewed. A control row without an evidence test is a gap, listed in the last section.

| Operation | Control (`data-control-id`) | Tauri command | Operator route | Authorization and fence | Receipt and recovery | Evidence |
|---|---|---|---|---|---|---|
| Start Runtime | `runtime-start` | `start_local_runtime` (`lib.rs`) | none (spawns `litecoworkd`) | OS user; single-instance lock in the daemon | `get_runtime_status` observation; restart yields a new incarnation | `tests/runtime-start-readiness.test.ts`, daemon `runtime_process_tests` |
| Refresh health | `runtime-refresh` | `get_runtime_status`, `get_runtime_lifecycle` (`runtime_lifecycle_bridge.rs`) | `GET /v1/runtime-lifecycle` | authenticated IPC peer; exact current incarnation or 503 | decoder rejects foreign identity | `tests/settings-runtime-health.test.mjs`, bridge decoder tests, `operator_ipc_tests` |
| Rename Workspace | `workspaces-rename-submit` | `rename_workspace` (unverified Tauri) | `PUT /v1/workspaces/{workspace_id}/name` | `If-Match` workspace version, `Idempotency-Key` | `workspace.renamed.v1`; conflict shows latest | `workspace_rename_operator_tests`, `workspace-rename` UI tests |
| Archive Workspace | `workspaces-archive-confirm` | `archive_workspace` (`lib.rs`) | Workspace archive route (`workspace_archive_routes`) | two-step confirm; expected version | read-only after receipt | `workspace_archive` storage tests, `settings-accessibility.test.mjs` |
| Change a local preference | `pref-theme`, `pref-density`, `pref-text-scale`, `pref-reduced-motion` | none (local) | none | webview localStorage, validated | applied at document root; corrupt value falls back | `tests/local-preferences.test.mjs` |
| Keep-awake | none yet | none yet | none yet (`RuntimeLifecycleView.power_assertion` not published) | n/a | `apps/litecoworkd/src/power_assertion.rs` state machine only | 9 unit tests; GAP: not wired |
| Preview stop | `runtime-stop-preview` | `get_runtime_stop_preview` (unverified Tauri) | `GET /v1/runtime-lifecycle/stop-preview` | authenticated IPC; exact incarnation | verdict BLOCKED or UNKNOWN today, never QUIESCENT | `runtime_stop_operator` tests, IPC test, `tests/runtime-stop-update.test.mjs` |
| Stop safely | none (text only) | none | `POST /v1/runtime-lifecycle/stop` is NOT mounted | n/a | n/a | GAP: 011-T13 request half blocked by 001/005/008 hooks |
| Check update quiescence | none (called by Feature 019 `install_update`) | `check_update_quiescence` (unverified Tauri) | `GET /v1/runtime-lifecycle/stop-preview` | same | `PositiveQuiescence` only on a clean QUIESCENT preview | bridge decoder tests |
| Update check and install | `UpdatePanel` (not mounted) | `check_for_update`, `install_update` do not exist | none | n/a | n/a | GAP: Feature 019 T11, B-43 |
| Agents: refresh, probe, bind, enable, disable, set default | `agent-catalog-refresh`, `agent-probe-codex`, `agent-probe-opencode`, `agent-binding-create`, `agent-binding-enable`, `agent-binding-disable`, `agent-binding-set-default`, `agent-default-clear-confirm` | `list_agent_installations`, `probe_agent_profile`, `create_agent_binding`, `enable_agent_binding`, `disable_agent_binding`, `set_workspace_default_agent_binding` (`lib.rs`) | `/v1/agent-profiles/probe` and the agent binding routes in `operator.rs` | Feature 002 owns authorization and versions | pending/receipt/error states | `tests/agent-catalog-control-map.test.mjs`, `agent-catalog-action-states.test.mjs`, `agent-catalog-settings.test.mjs` |
| Agents: registry | `agent-registry-refresh`, `agent-registry-search-submit`, `agent-registry-load-more` | `refresh_agent_registry`, `list_agent_registry` (`agent_registry_bridge.rs`) | Agent registry routes | Feature 002 | stale cache labelled | `agent-catalog-settings.test.mjs` |
| Runtime enrollment | `runtime-enroll`, `runtime-enrollment-refresh` | `enroll_local_runtime`, `list_workspace_runtime_bindings` (`lib.rs`) | `.../runtime-bindings/local-enrollment`, `.../runtime-bindings` | local principal | binding status shown | `tests/settings-merge.test.mjs` |
| Workspace instructions | `workspaces-instructions-save`, `workspaces-instructions-use-latest` | `create_workspace_instruction_revision` (`lib.rs`) | workspace instruction revision routes | expected revision; conflict returns latest | "Use this version" | `tests/settings-merge.test.mjs`, `workspace_instruction_bridge_tests` |
| Notifications link | `notifications-open` | none | none | n/a | navigates to the Notifications page | `settings-inert-control-audit.test.mjs` |

Open gaps recorded instead of documented as present: keep-awake wiring and UI, Stop safely, update check and install commands, and a browser run of any control above.
