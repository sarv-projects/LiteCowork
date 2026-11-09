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
