# DeepSeek Harness desktop shell: deep-dive (2026-11-10)

**Status:** research study. Not an owner contract. Authority stays with ARCHITECTURE.md, AGENTS.md and the accepted packages. Lessons map to proposed tasks (Section 14); no accepted package code is changed here.
**Source:** [deepseek-ai/deepseek-harness](https://github.com/deepseek-ai/deepseek-harness) `apps/desktop` at HEAD `fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159` (short `fcc0ad46`), cloned read-only into `repo_take/deepseek-harness` (gitignored, never imported, per AGENTS.md section 7). The target tree is 684 files (about 72,000 lines, 4.9 MB). No DeepSeek code is copied here; patterns are described and cited by file path and construct.
**Builds on:** [DEEPSEEK-HARNESS-PLUGIN-ARCHITECTURE-DEEP-DIVE-2026-11.md](DEEPSEEK-HARNESS-PLUGIN-ARCHITECTURE-DEEP-DIVE-2026-11.md) (A-58), which covered the plugin-manager package, the capability seam and profile bundles. This document does not repeat those findings; it covers the Electron shell that hosts that runtime. Where the two meet (profile ownership, recovery, bundled pnpm) the text says so and points back.

## 0. How to read this, and the honest limits of the read

The request was to read every folder, file and line. The depth actually achieved is stated here so no reader mistakes a survey for a proof.

| Area | Files | Lines | Depth of read |
|---|---|---|---|
| `README.md` (403 lines, very long paragraphs) | 1 (+ `README.zh.md` mirror, `README.i18n.yaml`) | 403 | Read in full, section by section. The Chinese mirror and the i18n hash record were checked structurally, not translated. |
| `docs/diagnostics`, `packaging`, `publishing`, `publishing/ci`, `windows-signing` guides | 15 (5 guides x EN, ZH, i18n yaml) | about 1,650 | English guides read in full except the last third of the packaging guide (macOS signing, runtime preparation) and the rollback section of CI publishing, which were skimmed. ZH and yaml mirrors not read line by line. |
| `src/` | 85 | 11,459 | Line-level read: `ipc.ts`, `preload-app.ts`, `web-document.ts`, `host-protocol.ts`, `host-process.ts`, `backend-controller.ts`, `single-instance.ts`, `update-coordinator.ts`, `update-schedule.ts`, `fatal-recovery.ts`, `operation-journal.ts`, `owned-directory.ts`, `platform-view.ts`, `runtime-tree.ts`, `core-package-set.ts`, first 1,000 lines of `main.ts`, most of `browser-guests.ts`. Doc-comment and signature read only: the other 60 files (`main.ts` lines 1000-1590, `mandatory-update-policy.ts`, `policy-test-auth.ts`, `keyboard.ts`, `file-drag*.ts`, `crash-report.ts`, `login-shell-environment.ts`, `command-*.ts`, `settings-file.ts`, `quit-confirmation.ts`, `mandatory-update-window.ts`, welcome and account files, all `preload-*.ts`). `locale.ts` (1,590 lines) is eight parallel message catalogs; read for structure and key typing, not word by word. |
| `renderer/` | 14 | 823 | HTML and CSP meta tags read; CSS/JS skimmed. |
| `scripts/` | 192 | 15,224 | Directory listing plus the leading doc comments of every non-`.d.mts` file (about 120 files); `electron-builder-config.mjs` (276), `runtime-file-policy.ts` (48) and `tsdown.config.ts` (113) read in full. The packaging, signing, notarization, publish and installed-update scripts were **not** read line by line. |
| `tests/` | 320 | 35,028 | `tests/README.md` read; all 181 `*.spec.ts` names listed (about 1,291 `it`/`test` cases counted by pattern); `main-startup.spec.ts` case titles read (50+ of its cases); `expected/` 59 files and `fixtures/` 31 files listed and a sample opened (`update-restart-win32-en-US.json`); not every expected file or fixture was read. Sampling of generated golden outputs is stated, not hidden. |
| `installer/` | 26 | 2,924 | Locale JSON structure and the README's installer chapter read; NSIS (`*.nsh`), C/C++ helper sources not read line by line. |
| `config/`, `cli/`, `resources/`, `windows-signing/` | about 30 | about 4,000 | Listed. Examples and thin launchers sampled. |
| Binary assets (listed, not opened) | 10 | n/a | `installer/assets/{brand,brand-2x,brand-dark,brand-dark-2x,uninstaller-sidebar}.png`, `resources/{icon,icon-macos,icon-windows}.png`, `resources/tray-windows.ico`, `tests/__snapshots__/preload-menu.client.spec.ts.snap` (text snapshot, 5 lines). |

Roughly 25% of lines were read at line level, another 40% at doc-comment/structure level, the remainder (mostly generated-style golden outputs, mirrors, NSIS and signing script bodies) were listed and sampled. Every claim below is tied to a file I did open. Claims that rest on README prose rather than code are marked **(README)**. A follow-up pass over `scripts/` signing and publish bodies is recommended before any of Section 11 is turned into a design (Section 15).

## 1. Directory map

`apps/desktop` (684 files). Counts are files / lines.

| Directory | Files / lines | Role |
|---|---|---|
| `src/` | 85 / 11,459 | Electron main-process shell, preloads, update coordinators, recovery, IPC. `main.ts` (1,590) is the composition root; `locale.ts` (1,590) holds eight message catalogs. |
| `src/client/` | 3 | The only React code: native Welcome/onboarding page, bundled to an IIFE by `tsdown.config.ts`. The product UI is not in this app. |
| `src/updates/electron-updater/` | 3 | Adapter pieces: channel selection, discarding a retired update, an HTTP executor with an idle deadline. |
| `renderer/` | 14 / 823 | Static shell-owned documents (update dialog, mandatory update frame, welcome, login-loading), each with a restrictive CSP meta tag. |
| `resources/` | 7 / 3,524 | Icons (PNG/SVG per platform) and the Windows tray `.ico`. |
| `cli/` | 3 / 37 | `dsh` / `dsh.cmd` launch scripts and a C link entry; reuse the app's Electron in Node mode. |
| `config/` | 10 | Example dotenv and JSON presets for macOS and Windows packaging and publishing (secrets never in repo). |
| `installer/` | 26 / 2,924 | NSIS theme and pages, a native GDI+ progress renderer (`window-frame.cpp`, 448 lines), uninstall-data helper, eight locale JSON files. |
| `packaging/`, `publishing/`, `publishing/ci/`, `windows-signing/`, `docs/diagnostics/` | 18 | Operator guides (EN, ZH, i18n pairing yaml). |
| `scripts/` | 192 / 15,224 | Build, runtime preparation, signing (Windows EV token, macOS notarization), packaging supervision, COS/EdgeOne publish, rollback, installed-update qualification, runner setup. |
| `tests/` | 320 / 35,028 | 181 specs, 59 `expected/` goldens, 31 `fixtures/`, `installed-update/`, `signing-concurrency/`, `updates/electron-updater/`. Test code is about 49% of the app's lines. |
| Root | 13 | `package.json`, `electron-builder.config.mjs` (5-line shim), `tsdown.config.ts`, three tsconfigs, READMEs, `.env.*.example`. |

The ratio is the first lesson: of 72k lines, roughly 11k are the shell itself; 15k are release engineering; 35k are tests and fixtures. A desktop shell that owns update, install and recovery is mostly verification and pipeline.

## 2. Architecture: processes, protocol, IPC

**Three processes plus a Host.** The Electron main process (`src/main.ts`) owns windows, protocol, updater and lifecycle. A **Host** is a child spawned from the same Electron binary in Node mode (`host-process.ts` spawns `process.execPath` with `--expose-internals`, `stdio: ['ignore','pipe','pipe','ipc']`, env built by `node-environment.ts`) running the full product backend (HTTP, WebSocket, agent runtime, plugins). The renderer is a sandboxed `BrowserWindow` loading the product web app.

**The shell is thin by design.** `README.md` "Key technical decisions" table: release identity is one unit (Electron shell, matching dsh runtime, pnpm), the web transport is shared with plain Web mode, plugin changes go through the same HTTP plugin manager as Web. There is no plugin-management IPC and no filesystem or shell access exposed to any renderer **(README, Installation ownership)**.

**Custom scheme, two origins.** `main.ts` registers `dsh-app` as standard, secure, fetch-capable, CORS-enabled, streaming, code-cached (`registerSchemesAsPrivileged`). `protocol.handle` serves `dsh-app://shell/` from packaged static assets with no Host dependency (so update and welcome documents work while the Host is down) and `dsh-app://app/` as: packaged web assets for a fixed allowlist of paths, otherwise a forwarded request to the Host, 503 until the Host and its cookie exist. `web-document.ts` `serveWebDocument` enforces GET/HEAD only, decodes the path, resolves it and refuses any target outside the root (path-containment), and injects a tiny script that creates a `Promise.withResolvers()` boot gate into `index.html` so the page waits for asynchronous Host injections.

**HTTP forwarding.** `forwardWebRequest` rejects a foreign `Origin`, rebuilds headers (dropping `host`, `origin`, `cookie`, `sec-fetch-site`), attaches the Host auth cookie itself, forwards with a streamed body and the request's abort signal, and strips connection-level response headers plus `set-cookie` so the cookie never reaches the page. Plugin bundle responses are rewritten to `no-store` because per-launch revisions would only accumulate in Chromium's disk cache. WebSockets get their `Origin` and cookie rewritten in `session.webRequest.onBeforeSendHeaders` only for the owned window and the Host authority, and cancelled if the origin is not `dsh-app://app` (`main.ts` ~line 700).

**Host IPC protocol.** `host-protocol.ts` carries `DESKTOP_HOST_PROTOCOL_VERSION = 4`. Host to shell events are a closed set validated by a hand-written guard (`isDesktopHostEvent` in `host-process.ts`): `starting{runId uuid}`, `ready{url, injections}`, `fatal{message, diagnostic}`, `platform-session`, `shutdown-complete`, plus correlated control responses `update-tasks`, `quit-inspection`, `file-drag-resource`. An invalid event is a protocol failure: the shell fails and `SIGTERM`s the child. `shutdown-complete` while not stopping is also a failure ("unrequested shutdown"). Shell to Host requests go through one `control()` method with a numeric request id, a per-request deadline (10 s update tasks, 2 s quit inspection, 12 s drag authorization), an `AbortSignal`, and cleanup of the pending map on every path.

**Renderer IPC.** `ipc.ts` holds the closed channel table `DESKTOP_IPC` (35 channels, all `dsh-desktop:`-prefixed) and `assertDesktopSender`, which requires the sender frame URL to be `dsh-app:` on an allowed hostname. `main.ts` `assertProductSender` is stricter: the sender must be the owned main window's `webContents`, the frame must be that window's `mainFrame`, and the URL must be the app origin. Boot, theme, locale, update and keyboard channels each re-check this. Preloads (`preload-app.ts`) use `contextBridge.exposeInMainWorld` only when `location` is the app origin; subframes and shell documents get an empty `dshDesktop`. Shell documents (update dialog, mandatory overlay) get their own isolated preloads and a private `MessageChannel` between preload and frame.

**LiteCowork mapping.**

| Pattern | Evidence | LiteCowork home | Verdict |
|---|---|---|---|
| Closed, versioned shell-to-daemon control protocol with guard-validated events and fail-closed on a malformed event | `host-protocol.ts`, `host-process.ts isDesktopHostEvent` | 011-T04 (IPC admission bounds), 011-T05 (runtime health) | **Adapt.** LiteCowork already has a typed Operator API over a named pipe / UDS; keep it, but add an explicit protocol-generation number in the readiness handshake and treat unknown event kinds as a fault, not a skip. |
| One control-request primitive with ids, deadlines, abort and unconditional map cleanup | `host-process.ts control()` | 011-T13 (authenticated stop), 011-T14 (stop/update UI), 019-T11 | **Adopt (design).** Every shell-initiated call that gates a destructive action carries its own deadline and a defined meaning when the deadline is missed (Section 5). |
| Loopback HTTP Host plus cookie plus custom scheme proxy | `main.ts protocol.handle`, `web-document.ts` | none | **Reject.** LiteCowork's daemon is reached over a named pipe (Windows DACL) or 0600 socket (011-T02/T03), not a TCP listener. That is a stronger peer-identity model than a shared cookie to 127.0.0.1. The Electron cookie-stripping, origin-rewriting and legacy-cookie cleanup code exists to compensate for the loopback choice. |
| Per-frame IPC admission: window, main frame, origin all checked on every call | `main.ts assertProductSender`, `ipc.ts assertDesktopSender` | 011-T04 ("frame isolation"), 019-T07 (peer-identity negatives) | **Adopt.** Tauri v2 equivalents: window label plus capability scoping per window, and a test that a non-main webview cannot invoke bridge commands. Add as negative cases in 019-T07. |
| Shell-owned static origin independent of the backend | `dsh-app://shell/`, `renderer/*.html` | 011-T15 (shell failure and recovery) | **Adopt.** The recovery, update and "daemon not ready" surfaces must render with the daemon down. LiteCowork's Tauri frontend is already bundled; keep recovery UI free of Operator API calls. |
| Index-head boot gate promise injected by the shell | `web-document.ts BOOT` | 011-T05 | **Adapt.** Equivalent: the UI shows a truthful "starting" state and the Tauri bridge resolves readiness only after the daemon handshake, never on a timer. |

## 3. Agent and session runtime (what the shell knows about it)

The desktop app contains **no agent loop**. It supervises a Host and asks it three questions through the control channel, and the answers drive the entire shell lifecycle.

1. **Admission lock and task inspection for update** (`updateTasks('inspect'|'lock'|'unlock')`, 10 s). "Active" means running agents including subagents and approval waits, queued messages, running or stopping jobs, and (for locking) admitted requests still draining **(README, Updates)**. API reads alone do not count as work. Locking returns 503 for new requests, drains admitted ones, then re-inspects, because an admitted write can create work.
2. **Quit inspection** (`inspectQuit()`, 2 s): `{activeTasks, scheduledTasks}`. Scheduled reminders armed in sessions loaded this run count; unloaded sessions are neither counted nor resumed **(README, Closing the window)**. `quit-confirmation.ts` maps the result to one of three explanations; a failed or late inspection is treated as "running tasks" ("unknown warns", it never quits silently).
3. **File-drag authorization** (`resolveDragResource`, 12 s): the Host confirms a session-scoped resource maps to a local file and returns size and a private digest; the shell checks device, inode, size and times before lending the file to the OS.

Tests prove the Host side separately: `tests/fixtures/host-update-qualification.mjs` boots the real profile loader and checks queued turns/steps, a running model request, pending questions/approvals, global and agent jobs in running/stopping states, admission lock without cancellation, restored admission, and rejected inspection after disposal **(tests/README.md, Evidence)**.

**LiteCowork mapping.**

| Pattern | LiteCowork home | Verdict |
|---|---|---|
| One shared definition of "interruptible work" consumed by update, quit, stop and keep-awake | 011-T12 (keep-awake defines "work runs" from 001 admitted-turn and 005 admitted-Attempt), 011-T13 (stop with settlement proof), 011-T14, 019-T11/T19, 010-T10 (window close / Runtime stop / sleep matrix) | **Adopt (new task P-31).** LiteCowork currently describes this predicate in three places (011-T12, T13, T14). DeepSeek's lesson is that the same facts must back four decisions, and "unknown" must resolve to "assume busy". Define one `QuiescenceReport` (admitted turns, admitted Attempts, running Terminal/Browser sessions from 008, armed Automations from 010, pending approvals) in the 011 contract and have 019's updater and the quit path call it. |
| Admission lock then drain then re-inspect, with unlock on every failure path | 011-T13, 019-T11 | **Adopt.** Maps onto LiteCowork's fenced stop (A-25). The extra step is the re-inspect after drain. |
| Scheduled reminders count toward the quit warning only when armed | 010-T10 | **Adopt.** A quit with armed Automations must warn: "Automations will not run while LiteCowork is closed" unless 010 proves host independence (B-26 accepted policy). |
| Host-side proof test with the real controller, only model and human answers faked | 011-T17, 019-T19 | **Adopt.** Matches the existing 019 "gate journey". |

## 4. Plugin and extension handling at the desktop layer

The plugin manager itself is covered in A-58. What `apps/desktop` adds:

- **Profile ownership.** Electron exclusively owns `$DSH_HOME/profiles/desktop` and takes the process-lifetime single-instance lock **before any profile access** (`single-instance.ts`, `claimDesktopSingleInstance`; `main.ts` claims before `main()`). CLI and Desktop share user data but never executable packages, lockfiles or `node_modules` **(README, State ownership)**. The installed `dsh plugin --profile desktop` command is allowed only while the app is quit and refuses an uninitialized profile.
- **Core packages are immutable and signed.** Core dsh and the Host live in `app.asar/dsh` with a `desktop-runtime.json` descriptor (`runtime-tree.ts`): schema version, release (shell, dsh, Host, Node, pnpm versions), platform, arch, shared package records, and a per-file `{path, bytes, sha256, executable}` inventory. `writeDesktopRuntimeAsync` seals it after materialization **and after native signing**; `verifyDesktopRuntime` (called from the electron-builder `afterPack` hook, `scripts/electron-builder-config.mjs`) recomputes the inventory with bounded concurrency (8 workers, drain-then-fail so no descriptor leaks) and fails the build on any difference. Startup deliberately does **not** re-verify; it reads metadata and checks shared-package records only **(README: re-adding startup comparison requires a concrete failure)**. Paths from metadata go through `runtimePath`, which refuses empty, absolute, backslash, colon, `.`, `..` segments.
- **Core package set.** `core-package-set.ts` defines a closed schema (`name` pattern, `version` pattern, `.tgz` file pattern, `sha512-` integrity) for tarballs of the release closure. First launch never installs core packages; the profile installs only external plugins.
- **Recovery that disables third-party plugins.** `fatal-recovery.ts` offers Exit, Restart, "Disable third-party plugins, back up profile patch, and restart", and "Get diagnostic report". The disable action calls the shared recovery function under the profile transaction lock and renames the profile `cordis.patch.yml` to a timestamped `.bak` **without parsing it**; the next start writes an empty patch; installed packages and earlier backups stay **(docs/diagnostics/README.md)**. `main.ts` wires `disablePlugins` to `DesktopProjectManager.disableAllPlugins()`. A known mandatory-update block refuses plugin mutations without stopping the Host **(README)**.
- **Pnpm and dependency runtimes are bundled** (primary-runtime: Node, pnpm, Python with a locked Office library set), installed offline on first use, version-bound to the Desktop release **(README, Key technical decisions)**.

**LiteCowork mapping.**

| Pattern | LiteCowork home | Verdict |
|---|---|---|
| Single-instance lock acquired before any state access; second launch focuses the owner | 011 shell; 011-T15 | **Adopt.** Verify the Tauri shell takes the single-instance lock before it spawns or attaches to litecoworkd, and that two shells cannot attach two UIs to one daemon with conflicting stop authority. Add as a negative test in 011-T17. |
| Immutable core, mutable extensions, separate trees | 013 (LiteSPM bridge), 002 (agent modules) | **Adapt.** LiteCowork's agents are native binaries installed by LiteSPM; the analogue is "LiteCowork never mutates the agent's own tree, only its own descriptor store" (002-T04). Already the design; this confirms it. |
| Sealed per-file hash inventory written after signing, verified at package time, not at every start | 019-T08/T09/T10/T14 | **Adopt (new task P-35).** Gives 019 a byte-level "what we shipped" artifact that the release evidence bundle (019-T15) can reference, and avoids a startup hashing cost. |
| "Disable all extensions and restart" as a first-class fatal-recovery action, with a backup of the configuration that was disabled | 011-T15, 013-T11, 002 | **Adopt.** LiteCowork needs the same escape hatch if a misbehaving AgentModule or connector prevents the daemon starting: a documented recovery verb that disables non-core modules without deleting them. |
| In-process plugins (host and plugins share one process) | none | **Reject.** LiteCowork's native agents are out-of-process across an ACP boundary (A-54). The DeepSeek recovery dialog exists because a bad plugin can crash the whole Host; LiteCowork's isolation makes that class of failure a per-agent fault. |
| Bundled Python/Node/pnpm runtimes shipped offline with a lock of wheel hashes | none in V1 | **Reject for V1.** Conflicts with the V1 packaging scope (019-T13 licensing and packaging decision). Note only as a precedent for hash-locked bundled interpreters if 007 professional artifacts ever need one. |

## 5. Settings and local state

Three independent, shell-owned stores, all outside the product's own configuration:

1. `userData/desktop/settings.json` (`settings-file.ts`, `settings.ts`, `update-settings.ts`). Read once at startup; reads never create the file; fields validate **independently** with Zod (an invalid popup flag does not invalidate a valid channel); unknown fields survive a read-modify-write; saves share a writer lock with startup initialization; write diagnostics distinguish "no write", "uncertain completion" and "committed, then lock cleanup failed". Channel changes go through IPC with a pending/applied/failed result, and an **uncertain** save suspends update operations until the user reloads (an explicit "Reload settings" action that never rewrites the file). **(README, Local desktop settings; `main.ts` updatesChannelSet handler.)**
2. `userData/keybindings.json` (`keybindings.ts`, `keyboard.ts`): device-local, separate from `DSH_HOME`, versioned, serialized single-writer, failed reads retain last accepted bindings and block edits, future-version files stay unchanged.
3. `userData/desktop/locale-cache` (`locale-cache.ts`): last confirmed UI language, so the native recovery dialog can speak the right language when the Host cannot start. Write failure never blocks.

Plus small markers: `background-close-confirmed` (acknowledgement under userData), `plugin-operations/operations*.jsonl` (Section 8), and `update-cleanup.jsonl`.

`README.md` states the ownership rule: shell-local settings are those that must be readable when the Host is not running; everything else belongs to Host configuration.

**LiteCowork mapping.**

| Pattern | LiteCowork home | Verdict |
|---|---|---|
| Rule: only what must be read with the daemon down lives in a shell store; everything else is daemon state | 011-T07 (applied and persisted local display preferences), 011-T05 | **Adopt.** LiteCowork already distinguishes shell preferences from daemon settings; write the criterion down in 011's contract so new settings are classified at review time. |
| Per-field independent validation, unknown fields preserved, reads never create | 011-T07 | **Adopt.** Prevents one bad field from resetting the rest. |
| "Uncertain write" is a distinct outcome that suspends dependent features instead of guessing | 011-T07, 019-T12 | **Adapt.** Use for update-channel and any setting that gates a risky operation. |
| Cached last-confirmed language for the pre-daemon surfaces | 011-T15 | **Adopt (small).** Needed by the recovery surface in any non-English UI. |
| Device-local shortcuts separate from synced/daemon settings, with revision checks | 011-T11 | **Adapt.** Fits LiteCowork's keyboard baseline; the conflict semantics belong in the shared shortcut owner doc if one is created. |

## 6. State management inside the shell

The shell uses explicit small state machines with generation counters instead of ad hoc flags.

- **`DesktopBackendController`** (`backend-controller.ts`): phases `starting | ready | error`; each start creates an `Attempt {cancelled, failure, host, cleanup}`; concurrent `start()` calls share one pending promise; `stop()` marks the attempt cancelled, awaits both its own cleanup and the pending start, and refuses new starts while stopping; `close()` is permanent; a failure after `ready` flips to `error` and still schedules cleanup; cleanup errors are aggregated with the original failure (`AggregateError`). Listener exceptions are caught and logged so a UI bug cannot break lifecycle.
- **`DesktopUpdateCoordinator`** (`update-coordinator.ts`): state `idle | checking | available | downloading | verifying | installing | ready | error` plus a `release` union `none | checked | downloaded | handed-off`. A **generation** counter is incremented on channel switch; every async continuation compares its captured generation and returns the current state if stale. `check/download/install/switchChannel` are single-flight (`??=` on an operation promise). `autoDownload = false` and `autoInstallOnAppQuit = false`: nothing downloads or installs without a user action. Install is `quitAndInstall(true, true)` only after `beforeRestart()` returns true; any exception or `error` event while installing becomes an `install` failure.
- **`DesktopUpdateSchedule`** (`update-schedule.ts`): base interval 10 min, exponential doubling on failure to a 1 h cap, uniform jitter within plus or minus 20%, floor 1 s, manual checks bypass the deadline and join an in-flight check, advance of the delay happens once per joined batch, monotonic clock (`performance.now`).
- **Presentation split**: `DesktopUpdateState` (full, main-owned, includes diagnostics) vs `DesktopUpdatePresentation` (semantic projection sent to renderers: phase, version, percent, failure kind, configuration error). The renderer never sees raw error text, URLs or authorization; failure copy is chosen by a typed `preparationFailure` kind so translations cannot reclassify a failure.

**LiteCowork mapping.**

| Pattern | LiteCowork home | Verdict |
|---|---|---|
| Attempt object with `cancelled`, cleanup promise, shared start, permanent close | 011-T05, 011-T15, 011-T20 (decompose App.tsx and Rust god files) | **Adopt (design).** A useful shape for the Tauri-side daemon supervisor: one start attempt at a time, cancellable, cleanup awaited before the next. |
| Generation token on async continuations | 019-T11/T12 (updater), 011-T20 | **Adopt.** Any "check, then download, then install" flow in 019 should be revocable by a channel or policy change without race. |
| Single-flight operation promises per verb | 019-T11 | **Adopt.** Prevents duplicate download/install under double clicks and resume events. |
| Typed failure kinds chosen by main, copy chosen by locale | 011-T06, 018 | **Adopt.** Matches LiteCowork's existing "truthfulness" rules: no raw error text in UI. |
| Update presentation is a read-only projection; UI cannot supply a version or authorize install | 019-T12 | **Adopt (key).** The "Update panel UI states" (019-T12) must never carry a UI-chosen artifact or approval; approval is a daemon/shell-held fact. |

## 7. Rendering and streaming UI

The shell itself renders very little: update dialogs, the mandatory overlay, welcome/onboarding, and the Windows caption menus. The product UI streams from the Host over the proxied HTTP/WebSocket endpoints. What is instructive:

- **Cold start ordering** **(README, Runtime and plugin activation)**: the main window loads the shared loading page offscreen from packaged assets *before* profile preparation and Host start; the login-shell environment read runs in parallel with profile preparation (`main.ts reconcileBackend` awaits `manager.applyRelease()` and `prepareHostEnvironment()` together); the page waits on the boot promise and then starts the client without navigating to another document, so there is one document for the whole session.
- **Hide, do not close.** Closing the main window hides it; the page, Host and tasks keep running and the same document (with drafts and scroll position) returns on show (Dock, second launch, `dsh://open`, tray). macOS leaves fullscreen first. A first-close acknowledgement on Windows is stored as a marker file.
- **Native overlays inside the window.** Update dialogs are `WebContentsView`s inside the owner (not separate OS windows), with input blocking tracked by revision so keyboard shortcuts and editing keys cannot reach the parent or browser guests while an overlay exists (`update-overlay.ts`, `keyboard.ts`). On Windows the mandatory overlay is an in-DOM shell frame whose actions travel over a private `MessageChannel`; the README is explicit that the shared DOM is **not** a security boundary and that product scripts can hide it but cannot clear main-process policy.
- **Native window material**: Windows 40-DIP caption with synchronized colors; macOS vibrancy with an opaque base while minimized or hidden to cover a known reattach gap (`main.ts` applyBackdrop). Reduce Motion, Reduce Transparency, Increase Contrast honored on the welcome window.
- **Streaming**: HTTP bodies and aborts are piped through unchanged; WebSockets connect directly to the Host with credentials injected by header rewriting rather than JavaScript.
- **Bounded native drag/drop**: file drag stages arbitrary bytes or Host-authorized resource references; total temporary + retained exports capped at 64 MiB, unused files released on pointer leave, OS-delivered copies expire after 60 s, system icons loaded asynchronously so large images are never decoded in main (**README**; `file-drag-files.ts`).

**LiteCowork mapping.**

| Pattern | LiteCowork home | Verdict |
|---|---|---|
| Show the shell immediately; start the backend in parallel with independent preparation; one document for the session | 011-T05, 019-T17 (performance and soak) | **Adopt.** Measure time-to-interactive and time-to-ready separately; ensure Tauri shows a truthful "starting" before the daemon handshake. |
| Close hides; work continues; reopen is the same document | 010-T10 (window close matrix), 011-T12 | **Adapt.** LiteCowork's tray/hide behavior must agree with 010's host-independence claim. If Automations require the daemon to run, the quit warning (Section 3) is the honest control. |
| In-window overlay for update/confirm instead of OS windows; input blocked by revision | 011-T14, 019-T12 | **Adapt.** In Tauri, prefer an in-webview modal bound to a daemon-held lock rather than a second webview; the principle (shell-held state, UI as projection) is what to keep. |
| macOS vibrancy opaque-base workaround | none (V1 is Windows and Linux) | **Reject (V1).** Platform-specific; macOS is out of V1 scope. |
| Header-injection for WebSocket auth | none | **Reject.** Not applicable to a pipe/UDS transport. |
| Byte budget (64 MiB) on staged drag data, release on leave | 003 (resource ingestion), composer drop | **Adapt, low priority.** Same budget discipline for dropped files in the composer. |
| Do not decode large images in the main process | 016, 007 | **Adopt (note).** Thumbnails and previews should be produced in a bounded worker. |

## 8. Crash recovery and diagnostics (high-value for 018 and 011)

This is the most directly reusable subsystem in the app.

- **One fatal dialog per process.** `DesktopFatalRecovery.report` sets `reported` once; the dialog shows a bounded tail (8 lines, 1,200 UTF-16 units, with a truncation notice and surrogate-pair repair) of the first error plus the crash-report status ("pending", "unavailable" or the path). `EADDRINUSE` is recognised and removes the plugin-disable option (disabling plugins cannot fix a port clash). The dialog loops: "Get diagnostic report" reveals the file and returns to the dialog; Exit/Restart/Disable stop the Host first and, if a recovery step fails, re-show the dialog with a recovery-failure headline instead of quitting.
- **Crash report file** (`crash-report.ts`): `crash-<UTC>-<source>.log`, source in `host | web-boot | renderer | main`, `phase` (`startup | running`), application and runtime versions, the error with enumerable properties and cause chain (capped at 256 KiB), the Host's own diagnostic (64 Ki tail of stderr plus startup metadata), a tail of renderer error-level console lines, owner-only permissions, the ten newest kept, older pruned by exact filename pattern only. The shell waits at most 1 s (`CRASH_REPORT_WAIT_MS`) for the report before opening the dialog; a slow disk shows "pending".
- **Operation journal** (`operation-journal.ts`): two JSONL segments (`operations.jsonl`, `operations.previous.jsonl`) of 256 KiB; records at most 4 KiB, no CR/LF; a queue capped at 64 KiB; **a partial trailing line is truncated at open**, an oversized segment is reset; any write failure disables the writer for the rest of the lifetime (no appending behind an uncertain partial write); `drain(timeoutMs)` is bounded (500 ms at exit, recovery and update) and a timeout aborts further writes. Records are validated and run-id matched in `host-process.ts` before enqueue.
- **Isolated, bounded report collection** (`operation-report-worker.ts`, `operation-report-job.ts`, `operation-history.ts`): after the basic report is saved, a `Worker` reads at most 64 KiB and 100 records from the two owned files, with a 500 ms budget (last 100 ms for cooperative cancellation before forced termination), one collector at a time, an explicit completion marker and omission counters. "A start without an end" is stated to not prove success.
- **Run identity**: the Host announces a `runId` (uuid) before profile loading; Host errors carry it as `hostRunId`; operation records are matched by it, so a report can distinguish "this run's records" from stale ones.
- **Update-install failure restores the Host** and keeps the downloaded target for retry (`main.ts publishUpdate`).
- **Update journal** (`update-journal.ts`): opt-in qualification evidence (`DSH_DESKTOP_UPDATE_JOURNAL_DIR`) that whitelists phase, version, integer progress, operation and a fixed error classification; no URLs, raw text, or unexpected fields survive.

**LiteCowork mapping.**

| Pattern | LiteCowork home | Verdict |
|---|---|---|
| One fatal surface per process with Exit / Restart / Disable non-core / Get report; EADDRINUSE special case; loop back to the dialog on a failed recovery step | 011-T15, 018-T08 | **Adopt.** Turns "Shell failure and recovery" into a concrete action set. LiteCowork's equivalent of EADDRINUSE is a stale pipe or socket path (011-T02/T03). |
| Crash record: bounded, owner-only, newest-N pruned by exact filename pattern, with source and phase fields | 018-T08 (local crash records), 018-T10 (support bundle, B-42) | **Adopt.** Add `source` (`daemon | shell | webview`) and `phase` (`startup | running | stopping`). The caps (256 KiB error section, 64 KiB stderr tail, 1 s wait) are a ready set of defaults. |
| Two-segment bounded JSONL with partial-line repair, disable-on-uncertain-write, bounded drain | 018-T07 (structured redacted log), 018-T08 | **Adopt (new task P-33).** Fits LiteCowork's "no unbounded logs" and secret-canary rules (017-T03). Records must remain redacted at the producer (018-T07). |
| Auxiliary evidence collected in an isolated worker under a deadline, never blocking the basic report | 018-T10 | **Adopt.** The support bundle must be able to ship a basic file even if enrichment hangs. |
| Run id announced before work begins and attached to errors | 018-T07/T08, 001 | **Adopt.** LiteCowork has operation ids; add a daemon run id to the readiness handshake and log lines. |
| Update-evidence journal that whitelists fields | 019-T15 (evidence bundle) | **Adopt.** Same discipline for release evidence: whitelisted fields only. |
| Reports that can contain plugin stderr, with a README warning | 018-T10 | **Adapt.** LiteCowork's preview (B-42) must redact agent stderr by default, not merely warn. |

## 9. Permissions and security (sandboxing, CSP, context isolation)

Window options (`main.ts createWindow`): `nodeIntegration: false`, `contextIsolation: true`, `sandbox: true`, `webSecurity: true`, `webviewTag` only for the primary window. `setWindowOpenHandler` denies and opens `http(s)` externally; `will-navigate` blocks anything that is not the app scheme or same-origin HTTP. Preloads are CJS bundles that may only `require` `electron`, `events`, `timers`, `url` (enforced by `scripts/desktop-bundle-imports.mjs`, a bundler plugin run from `tsdown.config.ts`, so an unresolvable bare import fails the build instead of failing at launch).

**Other controls found in code:**
- **Webview leases** (`browser-guests.ts`): a guest may attach only with a lease issued to its owner, a partition keyed by workspace identity, no Node integration and no guest preload; release closes the guest and awaits `destroyed`; leases from another window are rejected.
- **Embedded Platform document** (`platform-view.ts`): per-account `persist:` partition named by a SHA-256 of origin plus user id (anonymous otherwise), storage cleaned *before* load and after close (cookies, filesystem, IndexedDB, cache storage, service workers, shader cache, HTTP/auth caches), `setPermissionRequestHandler` returns false for everything, header injection only for the account origin and **removed on redirect to another origin**, `will-navigate`/`will-redirect` limited to the account origin without URL credentials, `will-attach-webview` prevented, bounds validated as finite integers 0..100,000, and the credential read is a synchronous IPC that never waits for network or disk and answers only the owned main frame on the configured origin.
- **Microphone** (`microphone-permissions.ts`): only the primary `dsh-app://app` frame; macOS entitlement and usage string packaged.
- **Directory picker and file actions** (`directory-picker.ts`, `file-actions.ts`): trusted frame only; clipboard-only remote image reads omit credentials, validate every redirect, reject the app's own Host and loopback aliases, stop on navigation, and are bounded (30 s, 64 MiB).
- **Process hygiene**: login-shell probe runs in its own process group and is killed on quit or timeout; `owned-directory.ts` refuses to follow junctions or symlinks when deleting (Electron's recursive remove would follow nested Windows junctions).
- **CSP on shell documents** (`renderer/*.html`): update dialog `default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'none'; img-src 'self'`; mandatory update `connect-src 'none'; img-src 'none'`; welcome `default-src 'none'` with explicit self sources, `base-uri 'none'`, `form-action 'none'`; the login-loading page allows only inline script/style because it is a packaged placeholder with no network. No CSP is set on the product app (it is served by the Host).
- **Electron fuse**: `electronFuses: { runAsNode: true }` in `electron-builder-config.mjs`. This is required by the Host design (Node-mode child of the same binary) and is a deliberate exposure; the README mitigates it with signed ASAR contents, hardened runtime and the sealed inventory.
- **Mandatory-update policy as a separate trust channel** (`mandatory-update-policy.ts`, README): policy origin is embedded at package time and ignored at runtime when packaged; HTTPS only (HTTP `127.0.0.1` only unpackaged anonymous); redirects rejected; page URL allowlist of exact origins; a known block survives transport, parse, and login failures and is cleared only by a fresh valid no-force response; the policy never selects an artifact or authorizes install.
- **Credentials**: the Host auth cookie is held by the shell and never in an Electron session; account tokens travel by private Node IPC; update-error details shown to the user contain only exit status, signal, acknowledgement and deadline facts, never plugin stderr **(README, Updates)**.

**LiteCowork mapping.**

| Pattern | LiteCowork home | Verdict |
|---|---|---|
| Sandboxed renderer, no Node, context isolation, window-open and navigation deny | 019-T06 (CSP and Tauri capability hardening audit) | **Adopt (checklist).** Convert to Tauri terms: no remote URLs in the main webview, explicit capability file per window, `shell:open` limited to `https`, navigation guard on the main webview. Confirm against the existing CSP in `apps/litecowork-ui/src-tauri/tauri.conf.json` (it already sets `script-src 'self'`, `object-src 'none'`, `base-uri 'none'`, `frame-ancestors 'none'`, `form-action 'none'`; `style-src` still allows `'unsafe-inline'`, which 019-T06 should either justify or remove). |
| Build-time check that every bare import in the shell bundle is resolvable in the packaged app | 019-T08/T14 | **Adapt.** The Rust/Tauri analogue is a packaged smoke that launches the installed build with a private data dir (see Section 11). |
| Webview/guest leases with owner binding and partition per workspace | 008-T04/T06/T19 (managed browser, preview iframe) | **Adopt (design).** The Browser Workbench needs the same "lease issued to this window, released explicitly, storage partitioned and cleared on close" model; also supports 008-T19 (remove `allow-same-origin` from the preview iframe). |
| Clear authentication storage before a document loads and after it closes; await cleanup before update install | 008-T04, 011-T14 | **Adopt.** Managed Chromium profiles must be cleaned or disposed on session end; the update path must await it. |
| Deny every permission request by default; allow named ones for the primary frame only | 008, 011-T11 | **Adopt.** Any webview that is not the main UI gets deny-all. |
| Never follow junctions when deleting owned directories | 017-T04 (Windows private directories), 005 | **Adopt.** The same hazard applies to LiteCowork's purge and restore staging on Windows. |
| Redirected subresources must not carry injected headers to another origin | 006 (connector egress), 008 | **Adopt (principle).** Any LiteCowork code that injects credentials into outbound requests must drop them on cross-origin redirect. |
| Separate trust channel for a forced update policy | 019 | **Reject for V1.** LiteCowork is local-first and has no first-party service to ask. A "known block survives failures" policy could matter for security advisories later; record only. |
| `runAsNode` fuse | none | **Reject.** Tauri/Rust does not need it. Keep it out of any future Electron-based side tool. |
| Platform account, Feishu test login, analytics intake | none | **Reject.** Out of scope for a local-first product and against the telemetry rules (018 local only). |

## 10. Auto-update

Mechanism: `electron-updater` with `autoDownload` and `autoInstallOnAppQuit` off, channels `stable` and `nightly` chosen independently of version suffixes, downgrade disabled, differential blockmaps, SHA-512 verified, Windows publisher check derived from the signing certificate's `CN`, `O`, `C` into the shipped `app-update.yml` (the downloaded feed does not choose the publisher). The shell adds:

- **User-driven only.** Automatic checks never open dialogs or download; a user picks "download", then a ready confirmation, then a second approval that stops tasks.
- **Quiescence gate** inside `beforeRestart` (`main.ts`): wait for command management and workspace recovery to settle; inspect tasks; fullscreen exit (with timeout kind); confirm; analytics intake with a 1 s deadline (not LiteCowork-relevant); lock admission; close Platform view and await its storage cleanup; dispose file-drag; stop the Host *requiring graceful exit*; drain the operation journal 500 ms; only then hand off. Failure after Host stop restores a Host, navigates the existing window to reload page-owned state, and keeps the artifact.
- **Graceful stop ladder** (`host-process.ts stop`): send `shutdown`; wait 10 s; `SIGTERM`; wait 5 s; `SIGKILL`; wait 5 s; if still alive, error. With `requireGraceful`, a forced kill or a missing `shutdown-complete` acknowledgement exits the child but **refuses installation** with a typed `DesktopHostUncleanExitError`.
- **HTTP idle deadline**: no response headers or no bytes for 60 s fails (`DesktopUpdateHttpExecutor`); no total-duration limit for active downloads.
- **Channel switching** is transactional: persist, bump generation, cancel token, await in-flight check and download, discard cache (`discard.ts`), set new channel, re-check; a cleanup failure suspends configuration; never auto-downgrade.
- **Cleanup log**: obsolete-file removal failures after eligibility is revoked go to `update-cleanup.jsonl` with time, operation, path and error, falling back to stderr.
- **Hidden-window rule**: a download that finishes while the window is hidden defers the install confirmation until shown; the mandatory flow uses taskbar attention or a Dock bounce plus one silent notification, never focus stealing (`update-attention.ts`).
- **Qualification layers** (tests/README.md): local loopback server with real Electron HTTP and `NsisUpdater`; channel tests; real-Electron mandatory renderer; Windows installed-update operator checklist with a journal; update-signature test using the real publisher verifier.

**LiteCowork mapping.**

| Pattern | LiteCowork home | Verdict |
|---|---|---|
| No auto-download, no auto-install on quit; two user approvals; task-aware second approval | 019-T11/T12, 011-T14 | **Adopt.** Aligns with B-43 "updater gated on Feature 011 quiescence". |
| Graceful stop ladder with an explicit refusal to install after a non-graceful exit | 011-T13, 019-T11/T19 | **Adopt (key).** LiteCowork's stop already requires "settlement proof"; add that an update install is *refused* if the daemon needed a forced kill or did not acknowledge teardown, and that failure restores a running daemon of the current version. |
| Restore the previous runtime when the installer fails to launch | 019-T11 | **Adopt.** Daemon restart after a failed handoff, with the artifact retained. |
| Check scheduling: 10 min base, jitter, backoff to 1 h, single-flight, resume/focus respect the same deadline, manual bypass | 019-T12 | **Adapt.** LiteCowork should choose longer defaults for a local-first app (e.g. hours) and expose them as settings, but keep the jitter, backoff and single-flight mechanics. |
| Channel switching cancels, discards and re-checks, never downgrades | 019-T12 | **Adopt if LiteCowork ships two channels.** Otherwise record as a design note. |
| Inactivity deadline, not total deadline, for downloads | 019-T11 | **Adopt.** |
| Publisher identity from the signing certificate, written into app-side config, not read from the feed | 019-T09/T11 | **Adopt.** Confirm Tauri's updater signature verification uses a key compiled into the app, never fetched. |
| Journal of update states with whitelisted fields for installed-update qualification | 019-T15, 019-T19 | **Adopt.** |
| Update ID and policy over a hosted service; EdgeOne/COS; staging percentage | none | **Reject (V1).** LiteCowork has no hosted infrastructure in V1 (019 scope). Staged rollout is a V1.1 consideration. |

## 11. Packaging, installer, signing, publishing

(README prose and the leading doc comments of `scripts/*`; script bodies for signing and publish were not read line by line, see Section 0.)

**Build structure.** `tsc -b` then `tsdown` bundles `main.js`, preloads (CJS), a worker and a command-manager entry; the Welcome page is built by Vite as an IIFE inside the tsdown `onSuccess`. The workspace tsdown pass must finish first because the main bundle inlines workspace devDependencies from their output; `desktop-bundle-imports` fails the build if a bare import could not resolve in the packaged app. `electron-builder` config is generated by `createElectronBuilderConfig(env, ...)` from one release environment, with hooks `beforeBuild` (compile the NSIS native helper, render installer strings), `beforePack` (compute Office and native asarUnpack patterns, validate policy), `afterPack` (write macOS updater config, **verify the sealed runtime inventory**, verify unpacked PE files) and `afterSign` (sign Windows PE files in the output, verify). Native binaries (`.node`, `.dll`, `.so`, `.exe`, `spawn-helper`, ripgrep) are `asarUnpack`ed.

**Release identity.** One version string covers shell, dsh runtime, Host and pnpm. Production uses the dsh base version exactly; test builds append `.YYYYMMDD.index` (or `-test.` for a stable base) in the Asia/Shanghai date; `--build-version auto` proposes the next free index by reading published objects, falling back to local artifacts; a build commit and dirty flag are recorded in manifest metadata; production uploads tag `desktop-v<version>` *after* artifacts are public and a tagging failure never fails a finished upload. Unsigned artifacts carry a filename suffix so they cannot pass for release artifacts. **(packaging/README.md)**

**Windows signing.** EV code-signing certificate on a hardware token, via a SafeNet-compatible SignTool, with: a supervised preflight that signs and verifies a throwaway probe before building; an on-disk **attempt interlock** (`%USERPROFILE%/.dsh-desktop-signing/attempt.json`) acquired before the token is touched and released only after a successful sign plus verification of the primary signature; `SignTool Error: No private key is available.` treated as fatal (stop all retries to avoid locking the token); timestamping completed afterward on isolated copies without credentials, three attempts with 1 s and 2 s delays, retried only on a normally-exited failure; a **content-addressed signature cache** shared across worktrees (policy-bound key, account-owned directory, no linked paths); PE discovery that reads headers of every regular file (including extensionless), rejects links and malformed `MZ` files, and runs trust checks in batches of 32 with at most four processes; a final PE-signature audit and a fresh-cache ASAR/Host smoke before the completion record is written. Signing credentials are withheld from build subprocesses.

**macOS.** Hardened runtime with entitlements (JIT for Node/V8 and, on x64, unsigned executable memory), per-file signature cache bound to certificate, tools, identifier and entitlements, a temporary signing keychain per invocation, notarization via separately timed `notarytool` phases, DMG notarized and stapled separately from the ZIP, a notarization-proxy transaction with recovery records. (V1 LiteCowork has no macOS row.)

**Windows installer.** Custom NSIS with a native GDI+ progress page: a single opaque pixel buffer, static art drawn once; directory **transaction** (stage then rename a complete directory), new destinations must be empty and non-empty destinations must be registered installations; drive roots rejected after normalization; per-user only; silent update waits up to 10 s for the old app to exit then stops with exit code 2; the uninstaller removes Electron user-data and cache but **never the product data home**, protects overlap with `DSH_HOME`, refuses protected Windows folders, requires a fixed local drive, unlinks reparse points without entering them; an update-triggered uninstall (`--updated`, `/KEEP_APP_DATA`) retains data. Per-language NSIS strings are generated from the eight locale JSON files and the native installer test builds each language.

**Packaging supervision and evidence.** Every package command writes `.desktop-build/packaging-runs/<id>/{run.json, events.jsonl (flushed before each operation), redacted stdout/stderr logs, result.json, fatal.json}`; a fatal signing failure makes the supervisor terminate the stage process tree; failed stages cannot start later ones. Toolchain preflight probes archive tools and the NSIS compiler *before* the first long stage (`desktop-toolchain-preflight.ts`). A completion record exists only after signing, notarization and runtime verification.

**Publishing.** Packaging does not upload and strips COS credentials from child processes. Upload validates the completion record, deployment, target, version, filenames, sizes and SHA-512 *before reading credentials*; **binaries and blockmaps first, then channel YAML**; historical objects retained; unknown write outcomes reconciled by reading remote bytes; conflicting same-name bytes stop the operation; a newer channel version blocks a stale upload; durable `plan.json` / `events.jsonl` / `result.json` per attempt, with PUT intent flushed before the request, no credentials or raw SDK errors recorded, audit failure stops later writes. Fixed-name "latest" installers are a separate upload with `Content-Disposition` and a CDN purge whose acceptance (not completion) is reported and never auto-retried. CI publication runs on the **original build machine** from retained artifacts (so the signed bytes never travel through a hosted runner), requires GitHub Environment approval, has per-channel machine locks, tags with a short-lived GitHub App token, and offers a read-only-preview channel rollback script that restores historical feeds without moving binaries or tags (`scripts/rollback-desktop-channel.ts`, `desktop-rollback.ts`). **(publishing/README.md, publishing/ci/README.md)**

**LiteCowork mapping.**

| Pattern | LiteCowork home | Verdict |
|---|---|---|
| Sealed per-file hash inventory after signing; verify at package time | 019-T08/T09/T10/T14 | **Adopt (P-35).** |
| Packaged-runtime smoke with a private data directory and no network or model | 019-T14, 011-T17 | **Adopt (P-35).** The installed build must boot, answer the readiness handshake and stop cleanly using a throwaway data root; assert it touched nothing in the real LiteCowork data directory. |
| Unsigned/debug artifacts named so they cannot pass for release artifacts; publication refuses them | 019-T14/T15/T23 | **Adopt.** Cheap and strong; fits the evidence bundle. |
| Signing attempt interlock that never auto-clears and treats key-unavailable as fatal | 019-T09 (Windows signing, B-43) | **Adapt.** If LiteCowork signs with a hardware token or cloud HSM, adopt the interlock and no-auto-retry rules; if it uses a software certificate in CI, adopt only "sign a probe first" and "verify before publishing". |
| Signature cache keyed by policy and bytes | 019-T09 | **Defer.** An optimization worth building only if signing time becomes a measured bottleneck (019-T17). |
| PE discovery that reads headers rather than trusting extensions; fail on malformed PE | 019-T08/T09 | **Adopt (checklist).** Same trap exists for Tauri bundles with sidecar binaries (litecoworkd and any shipped helper). |
| Installer directory transaction; refuse non-empty unregistered destinations; silent update waits and exit code 2 | 019-T08 (NSIS per-user, MSI) | **Adopt.** |
| Uninstaller never touches the data home; overlap protection; reparse-point-safe delete | 019-T08, 017 | **Adopt (key).** State it as an acceptance criterion: uninstall leaves LiteCowork user data intact by default; any "remove data" option is explicit and reparse-point safe. |
| Native installer UI helper compiled from C++ (GDI+) | none | **Reject.** Visual polish, high maintenance; use stock NSIS/MSI pages for V1. |
| Packaging run journal with flushed events and a completion record gating publication | 019-T15 | **Adopt.** The evidence bundle should be generated from such a journal, not reconstructed. |
| Credentials read only after plan validation; plan, events, result per publish; objects before feed; reconcile unknown outcomes; original-machine retention; short-lived token for tagging | 019-T10/T15 | **Adapt (P-36).** LiteCowork's V1 distribution endpoint is undecided in the docs I read; whatever it is, the order (artifacts, then manifest/feed), the pre-credential validation, and the unknown-write reconciliation are reusable. |
| Release feed rollback script restoring feeds only, read-only by default | 019-T23, 019-T15 | **Adopt (V1.1 or at go/no-go).** |
| COS, EdgeOne, Chinese-language approval summaries, Feishu | none | **Reject.** Region- and vendor-specific. |

## 12. Internationalization

- **Shell copy**: eight catalogs (`en, zh, ja, es, fr, de, ru, pt`) in one file (`locale.ts`), typed as `DesktopMessages = {[Key in keyof typeof en]: string}` so a missing key in any non-English catalog is a compile error; placeholders via `formatDesktopMessage`; ready-confirmation copy varies by platform.
- **Resolution order**: explicit saved language > first supported OS language > English; the main renderer receives the same inputs through an isolated preload before it mounts; the welcome window has no selector; automatic selection writes no preference. The last confirmed language is cached (Section 5).
- **Installer**: one JSON file per language (`installer/locales/*.json`, each with `nsisLanguage` and 40 string keys) is the single source; `scripts/installer-locales.mjs` validates keys, placeholder formats and command examples and generates NSIS strings; the native installer test installs and checks each configured language; `pnpm run check:i18n` and a repository UI i18n gate scan desktop sources **(README)**.
- **macOS**: `CFBundleLocalizations` declares all eight languages so the OS picks the initial language.
- **Docs**: every guide has `README.zh.md` plus `README.i18n.yaml`, a per-heading hash pair of the English and Chinese block outside code, re-recorded by a `verify-translation-pairing` command so a one-sided edit is detectable.
- **Golden copy**: user-visible strings in dialogs are pinned by `tests/expected/*-en-US.json`/`*-zh-CN.json` (e.g. `update-restart-*`, `command-management-*`, `fatal-dialog-*`), so a wording change is a visible diff.

**LiteCowork mapping.**

| Pattern | LiteCowork home | Verdict |
|---|---|---|
| Catalog typing makes a missing key a compile error | 011-T06/T11, 016 | **Adopt if/when LiteCowork localizes.** I found no i18n requirement in the V1 specs I sampled; this is a design note for when one is added. |
| Language preference resolution order with OS fallback and no write on automatic | 011-T07 | **Adopt (small).** |
| Installer strings generated from per-language JSON, validated, with a build test per language | 019-T08 | **Adopt only if more than one installer language ships.** |
| Doc translation pairing hashes | docs/ | **Reject.** LiteCowork docs are English-only. |
| Golden files for user-visible copy | 011, 019-T16 (accessibility) | **Adopt (P-37).** Strong guard against drifting from the "truthfulness" rules (011-T19): the exact words on stop, update and failure dialogs become reviewable. |

## 13. Testing and performance

**Testing.** 181 `*.spec.ts` (about 1,291 cases), `*.e2e.ts` for real Electron, 59 golden files, 31 fixtures. Notable properties:
- **Behavior pinned at the composition root.** `main-startup.spec.ts` (over 2,000 lines) drives `main.ts` with substituted Electron and asserts: settings persistence and failure suspension, mandatory block semantics, hide-on-close, tray confirmation, macOS shutdown skip, quit-confirmation races, update dialogs and manual-check feedback, foreign-renderer IPC rejection, cleanup ordering before install. Titles are behavior statements ("accepts product IPC only from the current application top frame in the owned main window", "hides the workspace before intentional Host shutdown can look like reconnection").
- **Fake clocks, injected dependencies**: `DesktopUpdateSchedule`, `OperationJournal`, `DesktopFatalRecovery`, `runtime-tree` and `run-tree` accept injected time, random, spawn and filesystem operations; constructors validate budgets (`RangeError`).
- **Real-Electron fixtures**: loopback artifact server, real `NsisUpdater`, real sandboxed preload and mandatory renderer, 17 scenarios with screenshots; separate theme (12 cases) and overlay-visibility fixtures; an interactive Windows runner.
- **Installed-update qualification** is operator-driven and evidence-bound: a checklist, per-run journal, two synthetic versions, a firewall fault-injection script with typed confirmation, and a documented list of what the harness does *not* prove (no real installer, no differential updates, no macOS).
- **Honesty about evidence.** The README repeatedly separates "package succeeded" from "installed and qualified", marks unverified items (macOS login, enterprise code-integrity policy, notification qualification) and ties every claim to the layer that produced it.

**Performance techniques.**
- Parallelize independent startup work (login-shell read, profile preparation); show the loading document before the Host exists.
- `codeCache: true` on the custom scheme; `no-store` on per-launch plugin bundles; async system icons for drag.
- Bounded concurrency with drain-then-fail semantics (runtime hashing: 8; PE header reads: 8 open files; signature checks: batches of 32, at most 4 processes; cache restore 4 default, 1..8; tarball packing 4 default); overlapping tarball packing with Electron and runtime preparation in separate output directories; LZMA (`ULMO`) DMG.
- Hard budgets everywhere: 64 MiB drag stage, 64 KiB journal queue, 4 KiB record, 256 KiB segment, 256 KiB error section, 1,200-character dialog detail, 500 ms shutdown drains, 2 s quit inspection, 10 s control requests, 60 s update idle.
- Native installer progress uses one reused pixel buffer and copies repainted regions only; repaint timings are diagnostics, not CI limits.
- Windows packaging CI normalizes process CPU/I/O/memory priority and refuses to treat a run with unavailable priority control as a performance comparison.

**LiteCowork mapping.**

| Pattern | LiteCowork home | Verdict |
|---|---|---|
| Composition-root tests with substituted platform APIs, naming behaviors as sentences | 011-T17, 011-T20 | **Adopt.** When App.tsx and Rust god files are decomposed (011-T20), keep a thin integration test per lifecycle behavior. |
| Injected clock/random/IO and constructor budget validation | 018-T13, 019-T18 | **Adopt (style).** Fits "failure, recovery and backpressure suite". |
| Operator-driven installed-update qualification with a whitelisted journal and an explicit "does not prove" list | 019-T14/T15/T19 | **Adopt.** The evidence bundle's USER sign-off should carry the same "not proven" list. |
| Fault-injection fixture (network block with typed confirmation) | 019-T18 | **Adopt (Windows).** For resilience matrix rows that need a network drop. |
| Bounded concurrency with drain-then-fail | 019-T17, 017-T14 | **Adopt (style).** |
| Do not claim a packaged build is qualified from a green unit run | 019-T22/T23, AGENTS.md | **Already LiteCowork policy;** this is independent confirmation. |
| Priority normalization of CI shells for comparable timings | 019-T17 | **Defer.** Only when soak numbers are compared across runs. |

## 14. Recommendation summary and proposed tasks

Adopt, adapt, reject at a glance (packages touched in brackets).

**Adopt (design or acceptance criteria):**
1. Shared interruptible-work report used by stop, update, quit, keep-awake; unknown means busy [001, 005, 008, 010, 011-T12/T13/T14, 019-T11/T19].
2. Update flow: no auto download/install; admission lock, drain, re-inspect; graceful-stop ladder; refuse install after unclean exit; restore daemon on failed handoff; UI is a projection, never supplies a version [011-T14, 019-T11/T12].
3. Single fatal recovery surface with Exit / Restart / Disable non-core modules / Get report; bounded crash report; run id; newest-N prune [011-T15, 018-T08/T10].
4. Bounded two-segment JSONL journal with partial-line repair and bounded drain; isolated auxiliary collector [018-T07/T08/T10].
5. Sealed runtime inventory and packaged smoke with private data root [019-T08/T09/T10/T14].
6. Uninstaller leaves user data intact; reparse-point-safe deletes; installer directory transaction [019-T08, 017-T04].
7. Unsigned/debug artifacts named distinctly and refused by publication [019-T14/T15].
8. Per-frame IPC admission and webview lease model; deny-all permissions on non-main webviews; cleanup before load [011-T04, 019-T06/T07, 008-T04/T06/T19].
9. Shell-local settings criterion, per-field validation, "uncertain write" state [011-T07].
10. Golden files for user-visible copy [011-T19, 019-T16].
11. Single-instance lock before any state access [011-T15/T17].

**Adapt:** protocol generation in the readiness handshake (011-T05); update check cadence (hours, not minutes) with jitter and backoff (019-T12); hide-on-close tied to 010's host independence (010-T10); signing interlock only where hardware tokens are used (019-T09); publication order and unknown-write reconciliation to whatever distribution endpoint LiteCowork picks (019-T10/T15); login-shell environment capture for agents launched from a Linux GUI (002-T07/T13, 005; see conflict note below).

**Reject:** loopback HTTP Host with cookie and custom-scheme proxy (LiteCowork uses pipe/UDS); in-process plugins; `runAsNode`; mandatory-update policy service, Feishu auth, analytics, Platform embedded account views; macOS vibrancy and notarization work (not in V1); native GDI+ installer renderer; COS/EdgeOne publishing; bundled Python/Node/pnpm runtimes (for V1); `dsh` PATH command installation and its ownership receipts (LiteCowork has no V1 CLI; the fingerprint-bound confirmation idea is already covered by 013-T08 approvals).

**Conflict note: login-shell environment vs environment scrubbing.** `login-shell-environment.ts` runs the user's login shell (from the user database, not `$SHELL`) as `-ilc`, reads `env -0` between delimiters under a process-group-killed 10 s timeout, falls back through `zsh`, `bash`, `sh`, ignores `PWD`, `OLDPWD`, `SHLVL`, `_`, and launcher-owned names, and merges the result *over* the inherited environment for the Host and everything it spawns. This fixes "GUI launch misses PATH", which will bite LiteCowork on Linux when a native agent such as OpenCode or Claude Code is launched from a desktop entry. But it conflicts with LiteCowork's rule that captured subprocesses receive a scrubbed environment (compare `scrubbedParentEnv` in A-58) and with 005 Environment boundaries. Resolution proposed: capture the login environment once in the shell/daemon, store only a **PATH-class allowlist** (PATH, locale, XDG dirs, toolchain homes), and never forward credentials from it. That needs an owner decision (it is the single place where this study asks to loosen a LiteCowork default), so it is a proposal, not a recommendation to merge.

**Proposed tasks (owner review, none created in the packages here).** Numbered after the highest existing proposal (P-30).

| ID | Proposal | Evidence | Touches |
|---|---|---|---|
| P-31 | Define one `QuiescenceReport` (admitted turns, admitted Attempts, running Terminal/Browser sessions, armed Automations, pending approvals; unknown = busy) and make stop, update, quit and keep-awake consume it. | `host-process.ts updateTasks/inspectQuit`, `quit-confirmation.ts`, README Updates | 011-T12/T13/T14, 019-T11/T19, 010-T10 |
| P-32 | Per-request deadline and defined meaning-on-timeout for every shell-to-daemon call that gates a destructive action; correlation ids; map cleanup on all paths. | `host-process.ts control()` | 011-T04/T13 |
| P-33 | Bounded rotating JSONL journal pattern (two segments, record cap, queue cap, partial-line repair, disable on uncertain write, bounded drain) and isolated deadline-bound auxiliary collector for the support bundle. | `operation-journal.ts`, `operation-history.ts`, `operation-report-job.ts` | 018-T07/T08/T10, 011-T15 |
| P-34 | Updater coordinator spec: generation token, single-flight verbs, no auto-download or install-on-quit, jittered backoff schedule, graceful-stop ladder with refuse-on-unclean-exit and restore-on-failed-handoff, UI as read-only projection. | `update-coordinator.ts`, `update-schedule.ts`, `host-process.ts stop()` | 019-T11/T12, 011-T14 |
| P-35 | Sealed per-file hash inventory written after signing and verified at package time; packaged smoke run against a private data root asserting no writes to the real data directory; distinct names for unsigned/debug artifacts. | `runtime-tree.ts`, `electron-builder-config.mjs afterPack`, `smoke-runtime.ts` (header) | 019-T08/T09/T10/T14/T15 |
| P-36 | Publication discipline: validate plan before reading credentials, artifacts before manifest, flushed intent events, reconcile unknown writes by reading back, retain signed artifacts on the build machine, feed-only rollback with read-only preview. | `publishing/README.md`, `publishing/ci/README.md` (script bodies not read) | 019-T10/T15/T23 |
| P-37 | Golden files for user-visible dialog copy on stop, update, recovery and failure paths, plus a key-parity check if localization is added. | `tests/expected/*`, `locale.ts` typing | 011-T19, 019-T16 |
| P-38 | Owner decision: login-shell PATH-class capture for agents launched from a Linux desktop entry (see conflict note). | `login-shell-environment.ts`, README Host environment | 002-T07/T13, 005 |

Existing proposals reinforced: P-27 (durable run-record and liveness fence; the DeepSeek Host `runId` plus operation journal is the shell-side counterpart), P-20 (capability seam; the shell's `DesktopBackendHost` interface shows the Definition/Provider split for the daemon supervisor).

## 15. What to read next, and what not to conclude

- A line-level pass over `scripts/` signing, notarization, upload-plan and publish-resume bodies before P-35 and P-36 become designs. The doc-comment level read is enough to show intent, not to confirm edge cases.
- `main.ts` lines 1,000-1,590 (menus, quit wiring, before-quit installer branch, window lifecycle) before P-31 and P-34 are specified.
- `crash-report.ts`, `login-shell-environment.ts`, `mandatory-update-policy.ts` and `keyboard.ts` in full before adopting any numeric default quoted here from the README.
- This study does not show that DeepSeek's update flow is correct on real machines: its own README lists unqualified items (macOS login, enterprise code-integrity policy, notification behavior, differential updates, installed macOS flow). Treat patterns as designs to qualify in LiteCowork's own matrix, not as proof.
- Licenses must be re-checked at the repository root before any code is adapted; this document adapts no code. The clone stays gitignored.
