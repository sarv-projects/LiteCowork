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

## Current implementation crosswalk (re-checked 2026-11-09)

- `WorkbenchPanel` is a React right pane with launcher, Files, Terminal, Browser and Review (`diffs`) paths. `WorkbenchTab` declares `activity` but no navigation or Activity render branch exists; the final `else` renders `ReviewPane`.
- Files is a list/filter of the Workspace Resource catalog; preview is fetched for an exact ResourceRevision. It is not a native filesystem tree or Environment browser.
- Browser is a URL input and iframe preview labelled "Embedded page preview". `browserUrl` starts empty (register A-51); there is no default endpoint and no containment claim. It is not a Chrome process/provider, managed profile, persistent session, browser agent or CDP/Playwright controller.
- Terminal (`TerminalPane`) renders "Terminal execution is unavailable" with the missing prerequisites. It has no input, executes nothing and shows no exit code, ConPTY, Job Object or KILL_ON_CLOSE label (register A-51; `tests/workbench-truthfulness.test.mjs`).
- Review (`ReviewPane`) renders "Nothing to review yet" and states that changes are never inferred from document text. No diff producer exists.
- Computer-use source has policy/consent helpers only; there is no Computer status surface in the Workbench.

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
