# DeepSeek Harness plugin architecture: line-level deep-dive (2026-11-09)

**Status:** research study. Not an owner contract. Authority stays with ARCHITECTURE.md, AGENTS.md and the accepted packages. Lessons map to proposed tasks (Section 8); no accepted package code is changed here.
**Source:** [deepseek-ai/deepseek-harness](https://github.com/deepseek-ai/deepseek-harness) at HEAD `fcc0ad46`, cloned read-only into `repo_take/deepseek-harness` (gitignored, never imported, per AGENTS.md section 7).
**Depth:** this is a line-level read of the plugin subsystem, not a skim. Files read in full: `packages/boot/plugin-manager/src/{index.ts (1,356 lines), operations.ts (663), install-spec.ts (123), build-approval.ts (96), git-transport.ts (72), run-tree.ts (77), registry.ts, patch.ts, versions.ts, failure.ts, install-failure.ts, official-install-target.ts, bundle-detail.ts, diagnostic.ts, github-connection.ts, tools.ts, types.ts (454)}` and the architecture notes `capability-seams`, `client-plugin-loading-model`, `profile-plugin-bundles`. Every claim below cites a file and the construct it comes from.

## 1. What the DeepSeek Harness plugin architecture actually is

It is a **profile-composed, pnpm-backed, Cordis-governed plugin system** with three layers that are cleanly separated:

1. **Composition layer (profiles and bundles).** A profile is a directory `$DSH_HOME/profiles/<name>` with a `package.json` carrying pnpm `dependencies` plus a `dsh.profile.bundles` ordered layer list, and a user `cordis.patch.yml`. A bundle is an npm package declaring `"dsh": { "bundle": { "patch": "./cordis.patch.yml" } }`. The tree composes over an empty root by applying each bundle's patch in `bundles` order, then the user layer and `--patch` overlays, through one shared `applyEntryPatches` call (`profile-plugin-bundles.md`). Resolution is two-anchored: `bundles` names resolve from the dsh installation first, then the profile directory, so in-box bundles are never pnpm-managed and always match the running binary.
2. **Lifecycle layer (plugin-manager over pnpm).** `PluginManager` (a `TypertRemoteService`, `index.ts:~330`) owns install/update/remove/exemption/approval through pnpm, with crash-safe locking, atomic rollback, and compatibility gating (Sections 2-5).
3. **Governance layer (Cordis DI).** The vendored `@cordisjs/plugin-loader` owns plugin lifecycle (mount, service-wait, teardown) on both host and browser; a consumer declares `inject: ['shell']` and its fiber stays PENDING until the provider exists (`capability-seams.md`, `client-plugin-loading-model.md`).

The key architectural idea, and the one most worth taking, is the **capability seam** (Section 6).

## 2. Crash-safe, recoverable package operations (operations.ts)

`runProfilePnpm` (`operations.ts:~270-470`) is the single most instructive function for LiteCowork's Feature 013 and 005 recovery work. What it does, in order, every step defensive:

- **Recorded-run fence.** Before reading any profile file it calls `activeRecordedRun(dir)` (`operations.ts:~180`), which reads `.plugin-manager/run.json` - a record a prior operation wrote naming the pnpm process tree it started. If that process ended but its pnpm tree is still alive (a lifecycle script outliving the parent), the new operation refuses with a precise diagnostic naming the pid, or, once the tree is gone, deletes the record and proceeds. This is exactly the "a crash between commit and dispatch must be recoverable" problem Feature 001 (A-05) and 005 face, solved with a durable intent record plus a liveness check.
- **Atomic rollback set.** It snapshots `package.json` and `pnpm-lock.yaml` (`savedFiles`) and the installed `node_modules/<dep>/package.json` for every dependency. On a compatibility rejection after pnpm already rewrote the tree, it restores those files and reinstalls the restored lockfile (`--frozen-lockfile`, or `--config.lockfile=false` when there was none) so the version that worked before keeps loading (`operations.ts:~400-440`). A failed mutation leaves the profile byte-identical - the same guarantee LiteSPM makes and LiteCowork 013 mirrors.
- **Pre-flight compatibility.** For every named install spec it reads the manifest without installing (`namedSpecManifest`, `operations.ts:~130` - a path spec from disk, a registry spec via `pnpm view ... engines peerDependencies --json`) and runs `evaluatePluginCompatibility`; an incompatible version is refused before pnpm runs so "the one already in use keeps working" (`operations.ts:~360`). LiteCowork's Agent Upgrade Shield (capability 2) should pre-flight the same way.
- **Idle-timeout tree kill.** A service run that captures output arms an idle timer; if pnpm prints nothing for `idleTimeoutMs` it kills the whole tree and reports `timedOut`, never hangs (`operations.ts:~300-330`). A run with inherited descriptors is never bounded (it would fire on a healthy interactive run).
- **Drain-after-exit.** After the process exits it drains the pipes under a fixed `DRAIN_AFTER_EXIT_MS = 2000` bound because a descendant that inherited the pipes can hold them open; it cuts the tail with a logged notice rather than waiting forever (`operations.ts:~340-380`).
- **Env scrubbing by execution mode.** A `service` run uses `scrubbedParentEnv()`; only a `cli` run inherits the real environment (`operations.ts:~290`). Secrets never reach a captured subprocess.

For LiteCowork: this is the reference implementation of "durable dispatch intent + atomic rollback + liveness-fenced recovery" that 001-T10 (outbox), 005 (Attempt admission) and 013 (install/apply) all need. The `run.json` + process-group liveness pattern is directly borrowable as a design (not code).

## 3. Process-group liveness (run-tree.ts)

`run-tree.ts` is 77 lines and worth copying as a design. A service run is spawned as its own process-group leader on POSIX (`leadsOwnGroup`, `run-tree.ts:~40`); `treeAlive` probes `process.kill(-pid, 0)` for the whole group, `pid` on Windows; `awaitTreeGone` polls every 15 ms up to 5 s (`run-tree.ts:~60-77`). All process ops are injectable (`RunTreeInternals`) so platform behavior is testable on any host. LiteCowork's Environment (005) and terminal (008) supervisors need exactly this group-liveness discipline for crash cleanup; it also matches LiteSPM's own Job-Object / Pdeathsig model documented in the LiteSPM study.

## 4. Build-approval with credential refusal (build-approval.ts)

When pnpm refuses to run a dependency's build scripts, `recordPendingBuilds` writes each blocked script name into `pnpm-workspace.yaml` `allowBuilds` as `"set this to true or false"`, and `approveBuilds` flips named ones to `true` only if still pending (a stale approval throws), without running anything (`build-approval.ts:~60-96`). The security detail LiteCowork must copy: `carriesCredential` (`build-approval.ts:~25`) refuses to record any `allowBuilds` key whose git URL carries a password or an HTTP(S) username, so a credential never lands in the profile settings or the pending list. This is the same "tokens never enter config or prompts" invariant LiteCowork holds (capability 8, AGENTS.md section 6), implemented at the package-manager layer. The approval-is-a-separate-decision-from-execution shape matches LiteCowork's plan-digest approval.

## 5. Install-spec parsing and Git transport (install-spec.ts, git-transport.ts)

- **`parseInstallSpec`** (`install-spec.ts:~70-115`) classifies a spec into `registry | path | tarball | git` before pnpm sees it, and refuses what neither pnpm nor the registry would take. A local path must be **absolute** - "the Host's working directory means nothing to the person typing into a browser, and a relative path resolved against the profile would point inside it" (`install-spec.ts:~75`). This is the exact threat model for LiteCowork's remote-control and agent-facing install surfaces: never resolve a path against an implicit cwd.
- **`dependencySpec`** (`install-spec.ts:~150`) strips HTTP user-info (`HTTP_USER_INFO`) from a recorded git URL when reconstructing the installable spec, so a stored dependency value never leaks a token.
- **`chooseGitTransport`** (`git-transport.ts:~60`) probes `git ls-remote` over HTTPS and, only if that fails, over an SSH form, with all interactive prompts disabled (`GIT_TERMINAL_PROMPT=0`, empty askpass, no controlling terminal) and `scrubbedParentEnv()`. LiteCowork's connector and LiteSPM egress work (006, 013) should pre-flight reachability the same fail-closed, non-interactive way.

## 6. The capability seam (capability-seams.md) - the headline lesson

A swappable capability is three roles that change at different rates:
- **Service Definition** - the Cordis service and vocabulary owning `ctx.<key>` (for example `dsh-shell`: `ShellExecutor`, `ShellRunResult`), never a bare interface.
- **Service Provider** - a plugin supplying an implementation (`dsh-bash-local`; `dsh-bash-sandbox` and remote providers are siblings behind the same definition).
- **Consumer** - what the model programs against (`dsh-tool-bash`: the `bash` tool schema), injecting the service key and never importing provider types.

The payoff stated in the note: "a sandboxed executor replaces `dsh-bash-local` without touching a tool schema." This is the structural answer to several LiteCowork couplings:
- The **ACP direction (A-54)**: the ACP client is the Service Definition; each native harness (OpenCode, Claude Code, Codex) is a Service Provider; the composer/timeline is the Consumer. A new harness never changes the composer contract.
- **Feature 005 Environments**: Linux bubblewrap and the future Windows AppContainer are sibling Service Providers behind one `Environment` definition (register B-17/B-35).
- **Feature 008 terminal/browser**: one PTY definition, platform providers (Linux PTY, Windows ConPTY); one browser definition, providers (managed Chromium, later attach-existing).

The note also warns about a real trap: `@cordisjs/plugin-capability` is a permission/security service, not a swap mechanism - do not conflate "capability" the security grant with "capability" the swappable implementation. LiteCowork already keeps these separate (Trust/grants vs provider adapters); the note confirms the separation is correct.

## 7. Client plugin loading and HMR (client-plugin-loading-model.md) - for the UI

The browser runs the same Cordis governance as the host over a bespoke module system (`dsh-client-modules`, `ClientModuleSystem`): a lazy CJS table where executing a bundle only **registers** a factory; side effects (including CSS injection) run at first `require`/materialization, memoized after. Plugins arrive as same-origin classic scripts addressed by package resources (`/plugins/??<a>/client.js,<b>/client.js&rev=<rev>`), with indexed source maps rewritten to repository paths so stacks reach TSX. HMR is one `client-hmr` plugin that stat-polls bundle artifacts (inotify does not fire on the network mount), then on the browser side does invalidate -> prefetch -> registry.delete -> drain disposers -> remove styles -> materialize -> `entry.refresh()`.

For LiteCowork this is a reference for a future extension-UI story (V1.1 extension creator), not V1: it shows how to load third-party UI safely with one governance model, revisioned immutable delivery, and no `eval`/inline-script (the module table is the terminal mechanism, import-maps and `eval` explicitly rejected). It reinforces the LiteCowork rule that rich/extension UI uses registered components and typed delivery, never arbitrary model HTML (capability 13, Feature 016).

## 8. What LiteCowork should take, mapped to packages

| DeepSeek pattern | Evidence | LiteCowork home |
|---|---|---|
| Durable run-record + process-group liveness fence for crash-safe operations | `operations.ts activeRecordedRun`, `run-tree.ts` | 001-T10 (outbox), 005 (Attempt admission/recovery), 013 (install/apply recovery) |
| Atomic rollback of manifest+lock on a rejected mutation, reinstall restored lockfile | `operations.ts runProfilePnpm restore/repair` | 013 (mirror LiteSPM byte-exact restore), 005 (Effect rollback) |
| Pre-flight compatibility read before install; keep the working version if the new one is incompatible | `operations.ts namedSpecManifest`, `evaluatePluginCompatibility` | 002 Agent Upgrade Shield (P-21), 013 |
| Credential refusal in build-approval and dependency-spec reconstruction | `build-approval.ts carriesCredential`, `install-spec.ts HTTP_USER_INFO` | 013, 006 (tokens never in config/prompts) |
| Absolute-path-only install specs; never resolve against an implicit cwd | `install-spec.ts parseInstallSpec` | 013 agent-facing install, future remote-control |
| Non-interactive fail-closed reachability probe before a network operation | `git-transport.ts readable/chooseGitTransport` | 006, 013 |
| Capability seam (Definition/Provider/Consumer) | `capability-seams.md` | A-54 (ACP); 005 Environments; 008 terminal/browser (P-20) |
| Lazy-factory, Cordis-governed, no-eval extension UI with revisioned delivery and HMR | `client-plugin-loading-model.md` | Future extension-UI (V1.1); reinforces 016 no-model-HTML |
| Profile = ordered bundle layers composed over an empty root | `profile-plugin-bundles.md` | A roadmap reference for LiteCowork Workspace/Coworker composition; not a V1 change |

Proposed tasks (owner review): P-20 (capability-seam ADR, already proposed) is reinforced; a new **P-27** records the durable-run-record + process-group-liveness recovery pattern as the reference design for 001-T10 / 005 recovery and 013 apply-recovery, and the credential-refusal and absolute-path rules as acceptance checks on 013's install surface.

## 9. Honest limits

This is a deep read of the plugin subsystem (every file in `plugin-manager/src`, plus the three governing architecture notes), cited by construct. I did not read all 20,665 files in the repo (most are unrelated: the agent loop, providers, web client shell, benchmarks, localized docs). No DeepSeek code was copied into LiteCowork; the clone stays gitignored. Licenses must be re-checked at the repo root before adapting any pattern into code; this document adapts no code.
