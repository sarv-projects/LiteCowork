# UI behavior - Workbench Browser Terminal and Computer

**Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.

Behavior, not pixels. Screen inventory: [V1-UI-SCREEN-CONTRACT.md](../../../docs/V1-UI-SCREEN-CONTRACT.md), [DESKTOP-SHELL-LLD.md](../../../docs/DESKTOP-SHELL-LLD.md), [CHROME-BROWSER-LLD.md](../../../docs/CHROME-BROWSER-LLD.md), and [V1-INTEGRATED-TERMINAL-LLD.md](../../../docs/V1-INTEGRATED-TERMINAL-LLD.md).

## Target surface

- Workbench is a resizable right-side view over authorized Workspace/Resource/Environment objects. It does not own privileged shell/browser processes. Closing the pane detaches the view unless the user explicitly requests a separate session Stop/Close operation.
- Files show admitted Resources or exact Environment filesystem views; labels and location identity distinguish Workspace Resource projections from live local files. A Resource preview is never labeled as a live filesystem editor.
- Browser shows provider availability, mode, engine/version, profile identity, session/tab state, current URL, permission and control owner/epoch. Managed browser and Connect Existing Chrome are distinct modes. A generic iframe preview, if retained, is labeled as embedded preview and has no agent-action or Chrome assurance badge.
- Take control/Return to agent are available only for a live provider-backed lease. UI displays the exact owner/epoch and pending-action state; Takeover disables queued agent input only after the provider confirms the new epoch; Return requires confirmation and fresh observation. Closing the side pane is separate from closing the browser or stopping a Task.
- Terminal tabs show exact shell profile/version, executable identity, current directory, Environment/process identity, containment evidence, session owner, and connection/lost state. Input/stdout/stderr and resize use a real PTY/ConPTY bridge. A tab switch, pane resize or detach/reattach does not own or restart the process.
- Explicit Stop is a separate command with process-tree/quiescence receipt. UI close is a view detach; daemon stop/restart renders ACTIVE→LOST/RECONNECTING only from authoritative service state. There is no silent shell restart.
- Diffs show actual content/revision or VCS comparison, exact source IDs/digests and true added/removed counts. Activity shows committed event/Attempt/Effect/process observations only, or is deferred to Work/Task detail by the screen contract.
- Computer use is only shown when a real provider and OS permission exists. Otherwise the UI states unavailable/unsupported with a reason; it never animates a phantom cursor or claims screen capture/input injection.

## Current implementation crosswalk (re-checked 2026-11-11)

- `WorkbenchPanel` is a React right pane with launcher, Files, Terminal, Browser, Review (`diffs`) and Activity paths. Activity has its own launcher/tab and status-only empty state; it does not fall through to Review.
- Files is a list/filter of the Workspace Resource catalog; preview is fetched for an exact ResourceRevision. It is not a native filesystem tree or Environment browser.
- Browser is a URL input and iframe preview labelled "Embedded page preview". `browserUrl` starts empty (register A-51); there is no default endpoint and no containment claim. It is not a Chrome process/provider, managed profile, persistent session, browser agent or CDP/Playwright controller.
- Terminal (`TerminalPane`) renders "Terminal execution is unavailable" with the missing prerequisites. It has no input, executes nothing and shows no exit code, ConPTY, Job Object or KILL_ON_CLOSE label (register A-51; `tests/workbench-truthfulness.test.mjs`).
- Review (`ReviewPane`) renders "Nothing to review yet" and states that changes are never inferred from document text. No diff producer exists.
- Computer-use source has policy/consent helpers only; the Workbench shows a status-only UNAVAILABLE message with no action control.

## Accepted V1 state table (2026-11-09)

| View | Windows x64 row | Linux x64 row | Loading / empty / error / denied / offline / stale |
|---|---|---|---|
| Browser | MANAGED_CHROMIUM when provider READY; otherwise NEEDS_SETUP/UNAVAILABLE with reason | Same | Starting spinner with cancel; empty tab list; provider error with repair; consent denied; OFFLINE when Runtime down; stale epoch shown as "control changed, refresh" |
| Terminal | UNAVAILABLE: "needs a qualified Windows Environment provider (Feature 005 B-35)" | Real PTY under admitted bubblewrap Environment | Discovering profiles; no profiles; spawn failed; Environment not admitted; LOST/RECONNECTING after restart; never a guessed exit status |
| Activity | Committed records for the bound Conversation/Task | Same | Loading; "No recorded activity yet"; read error with retry; stale cursor reload |
| Diffs | Exact ResourceRevision or VCS pair | Same | Computing; "No changes"; TOO_LARGE / BINARY / SOURCE_MISSING typed states |
| Computer | UNAVAILABLE (B-32 deferred), status text only, no control | Same | Not applicable |

Every unavailable state is a focusable status region with an accessible name and the concrete prerequisite; keyboard users reach each view through the launcher and tool pills.

## Required rendered regression cases after implementation

- Browser provider unavailable/needs setup/offline/ready; no iframe fallback masquerades as a qualified browser session.
- Managed profile isolation and connect-existing Chrome consent/version mismatch/revoke; default Chrome profile always refused.
- Browser stale session/tab/control epoch, input in-flight takeover, provider crash, fresh-observation Return and uncertain navigation form submit.
- Terminal profile discovery must not run startup scripts; selected shell exact executable/argv/cwd/env; unsupported shell remains unavailable.
- Terminal multi-tab attach/detach, resize, backpressure/high-volume output, explicit stop, process-tree containment, owner switch, pane close vs Stop, workspace change and daemon restart.
- Actual diff with equal revisions, rename, binary/unpreviewable content, no changes, missing source and uncommitted source; never report fabricated counts.
- Computer-use permission denied/revoked, sensitive-region exclusion, provider unavailable, per-action scope and stale permission.

## Validation

Source-level policy helper tests do not qualify a browser provider, terminal process, or computer-use backend. Qualification requires the actual OS provider and USER flows on the declared GA OS/build, with event-backed receipts and crash/restart tests.

## Accepted observation and takeover interaction

The repository owner requested a visible, low-overhead view and an immediate manual takeover path on 2026-11-11. Resolve this within accepted B-22/B-32 boundaries:

- When the real managed-browser provider is available, show only that browser tab's viewport in Workbench using ephemeral frames, a bounded latest-frame queue, and stale-frame labeling. Do not persist frames into Conversation or Activity by default. Label it **Browser view**; it is not a share of the laptop desktop. Whole-desktop capture/input remains unavailable under B-32.
- While a live browser lease is AGENT-owned, show a persistent, non-modal banner with session/tab identity and **Take over**. The command fences new agent inputs, resolves or cancels queued old-epoch inputs, reaches a bounded safe point for dispatched input, then atomically advances the lease epoch to USER. Report USER control only after the provider confirms the epoch and rejects old-epoch input. Reconcile a dispatched action that may already have taken effect; unresolved outcomes remain pending/unknown. Move focus to the browser after confirmed transfer.
- Browser **Take over** transfers browser input authority only. It does not stop Conversation generation, cancel a Task, or close the browser. Conversation **Take control** is a separate action owned by Feature 012: cancel the active Conversation turn through Feature 001 Stop, wait for observed quiescence, settle `CANCELLED` and record `OWNER_TOOK_CONTROL`; only then does the owner continue by hand. Sending a message is a separate subsequent turn, never an implicit takeover side effect.
- Without a served provider-backed lease, show status only. Returning control is separate and explicit: confirm, advance the epoch, obtain a fresh observation, and resume only from that observation. Never replay stale queued inputs or auto-submit a user-edited form.
