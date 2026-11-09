# Rival deep-dive: plugin architectures and capability patterns (2026-11-09)

**Status:** research and planning synthesis. Not an owner contract. Authority stays with ARCHITECTURE.md, AGENTS.md and the accepted packages. New items are proposed tasks for owner review (Section 9); no accepted package code is changed by this document.
**Scope:** eight rival repositories cloned read-only into `repo_take/` (research only, never imported, per AGENTS.md section 7), read at the architecture level: their plugin and extension systems, capability models, coding cockpit, interactive UI and the stack-comparable daemons. This answers the request to understand the DeepSeek Harness plugin-based architecture and to mine seven more rivals (including nexu-io/open-design) for capabilities to add.
**Honesty note on depth:** I did not read every line of every repo (the eight total roughly 1.9 million lines across ~32,000 files, mostly lockfiles, assets, tests and generated code). I read the architecture-defining source and design docs in each, line by line where they matter, and cite exact files. The two small repos (Open-Claude-Cowork, kuse_cowork) were read near-whole.

## Repositories and pins

| Repo | HEAD | License | What it is | Stack |
|---|---|---|---|---|
| [deepseek-ai/deepseek-harness](https://github.com/deepseek-ai/deepseek-harness) | `fcc0ad46` | see root | Agent harness with a full plugin manager and a capability-seam architecture | TS (Cordis DI), 20,665 files |
| [zai-org/ZCode](https://github.com/zai-org/ZCode) | `a9fe401e` | see root | Coding agent with an MCP-Apps interactive UI-plugin system | TS, 17,237 files |
| [different-ai/openwork](https://github.com/different-ai/openwork) | `5493b18` | see root | Work agent with an extension registry and manifest foundation | TS, 7,174 files |
| [DevAgentForge/Open-Claude-Cowork](https://github.com/DevAgentForge/Open-Claude-Cowork) | `7a9f99f` | see root | Minimal Electron Claude-Cowork alternative | TS/Electron, 53 files |
| [kuse-ai/kuse_cowork](https://github.com/kuse-ai/kuse_cowork) | `09c6ab0` | see root | Rust+Tauri local-first cowork agent, BYOK | Rust+Tauri, 130 files |
| [nimbalyst/nimbalyst](https://github.com/nimbalyst/nimbalyst) | `826adb4` | see root | Agent desktop with a Claude-plugin packaging surface | TS, 9,411 files |
| [paradevai/paradev](https://github.com/paradevai/paradev) | `a865880d` | see root | Desktop for running CLI coding agents in parallel, each in its own Git worktree | TS/Electron, 15,480 files |
| [nexu-io/open-design](https://github.com/nexu-io/open-design) | `07a86b1` | Apache-2.0 | Collaborative AI design-agent workspace with a daemon plugin registry and capability atoms | TS daemon, 14,266 files |

## 1. The headline pattern: DeepSeek Harness capability seams

The single most valuable idea across all eight repos is DeepSeek Harness's **capability seam** (`.agents/notes/implemented/architecture/2026-06-13-capability-seams.md`). A swappable capability is split into three roles that change at different rates:

1. **Service Definition** - the contract and vocabulary that owns `ctx.<key>` (for example `dsh-shell`: `ShellExecutor`, `ShellRunResult`). Never a bare interface; a registered service.
2. **Service Provider** - a plugin that supplies an implementation (`dsh-bash-local`, and sibling `dsh-bash-sandbox` or a remote provider against the same definition).
3. **Consumer** - what the model and other plugins program against (`dsh-tool-bash`: the `bash` tool schema). Consumers inject the service key and never import provider-specific types.

The payoff: a sandboxed executor replaces the local one without churning the tool schema the model sees. This is exactly the discipline LiteCowork needs for its native-agent adapters, Environment providers and terminal/browser providers. It validates the ACP direction (register A-54): the ACP client is the Service Definition, each native harness is a Service Provider, and the composer/timeline is the Consumer. It also tells us how to structure Feature 005's Environment providers (Linux bubblewrap and the future Windows AppContainer are sibling Service Providers behind one definition) and Feature 008's terminal/browser (one PTY definition, platform providers). DeepSeek pairs this with Cordis dependency injection (`inject: ['shell']`, a consumer fiber pends until the provider exists) - a pattern LiteCowork already approximates with its Operator service wiring, so the lesson is to make the seam explicit, not to adopt Cordis.

## 2. Plugin lifecycle: DeepSeek plugin-manager and open-design registry

Two repos implement a real, safe plugin lifecycle that LiteCowork's Feature 013 (LiteSPM bridge) and Discover can learn from, and both reinforce the LiteSPM model already pinned.

**DeepSeek `packages/boot/plugin-manager`** (`index.ts` ~900 lines plus operations/registry/patch/build-approval/git-transport) does profile-scoped plugin composition with:
- Atomic install with a file lock, a restore set (`package.json`, `pnpm-lock.yaml`) that is rolled back on failure or cancellation, and an install-control object with `installing`/`applying` phases where `applying` cannot be cancelled (`index.ts` `InstallControl`, `RESTORED_FILES`).
- Version-compatibility exemptions: a plugin incompatible with the runtime is refused unless the user grants an explicit `setVersionExemption(..., acceptRisk=true)` (`index.ts` `@Remote setVersionExemption`). This is a clean model for LiteCowork's "an agent upgrade must not silently break a binding" (capability 2, Agent Upgrade Shield).
- Build-approval gating (`build-approval.ts` `approveBuilds`/`recordPendingBuilds`): a package that wants to run build scripts is held until the owner approves - the same shape as LiteCowork's plan-digest approval.
- Multi-registry fallback, GitHub/Git transport pre-flight probing with timeouts, and protected management bundles that cannot be removed.

**open-design `apps/daemon/src/plugins/registry.ts`** is the closest match to what LiteCowork's Discover + Coworker-scope needs, and it is Apache-2.0:
- One manifest model fed by three sources with a documented compatibility matrix: an `open-design.json` sidecar, an adapted `SKILL.md`, or an adapted `.claude-plugin/plugin.json` (`adaptAgentSkill`, `adaptClaudePlugin`, `mergeManifests`, `validateSafe`). LiteCowork's LiteSPM/skills story should adopt the same "accept SKILL.md and .claude-plugin natively, normalize to one manifest" approach rather than inventing a format.
- **Trust tiers bound to source kind:** a `local` install is implicitly trusted (the user copied it); everything else starts `restricted` until an explicit `od plugin trust` flip (`resolvePluginFolder`, `defaultTrustForRecord`, `resolveCapabilitiesGranted`). Capabilities granted derive from `(manifest, trust)`. This is precisely LiteCowork's "installed is not authorized" rule (capability 7) made concrete.
- **Workspace-scoped visibility** (`pluginVisibleFromWorkspace`): bundled plugins are global; user plugins are quarantined from an explicit Workspace unless a Team or Personal binding grants it, with a careful `undefined` vs `null` scope distinction and an activation fence that survives async hub reads. This is a strong reference for LiteCowork Coworker/Workspace-scoped capability grants (capability 5, 11) with the same fail-closed discipline.

## 3. Interactive UI done right: ZCode UI Plugin (MCP Apps)

ZCode's `UI_PLUGIN.md` is the best reference for LiteCowork capability 13 (Intelligent UI and Interactive Artifacts) and the co-edit loop (Feature 012, 016). It builds interactive pages on the official `@modelcontextprotocol/ext-apps` SDK (MCP Apps), not bespoke HTML injection:
- A tool result can carry an interactive page via `_meta.ui.resourceUri` + an HTML resource; a conversation-side panel is declared by manifest `ui.surfaces`.
- The Excalidraw example is the exact co-edit pattern: the agent draws an editable canvas, the user edits it, the user **references a selection back into the conversation** (`updateModelContext()` / `sendMessage()`), and the agent makes local edits by stable element id (`read_scene` -> `apply_operations`) sharing one undo/redo with the user. Documents have revisions and a saved state, and pages are preserved across tab switches (the "active page kept" rule).
- Hard scope line: interactive pages are ZCode Desktop local-workspace only; web/mobile/remote keep plain MCP tool records. That matches LiteCowork's local-first V1 boundary.

The lessons for LiteCowork: build Intelligent UI on MCP Apps + typed resources with host-rendered components (which the accepted 016 design and the P-03 renderer already head toward), make rich artifacts editable documents with revisions (Feature 007/012), and support "reference this selection back to chat" as a first-class co-edit affordance (Feature 012 take-control/side-note already anticipates this).

## 4. Parallel coding cockpit: paradev

paradev is directly on-point for capability 3 and proposed task P-13: "a desktop app for running CLI coding agents in parallel. Each task gets its own Git worktree, branch, and terminals." It also ships a real browser extension host (`apps/desktop/src/main/lib/browser/extension-host/` with CRX3 loading, declarative-net-request engine, DNR host) - a reference for a real managed browser (Feature 008 B-22) rather than an iframe. The worktree-per-task model is exactly what Feature 005 (isolated Task execution) plus the coding cockpit should present: parallel agents, separate worktrees, separate terminals, aggregate diff review. paradev confirms the shape; LiteCowork keeps its own Task/Attempt/Environment authority around it.

## 5. Stack twins and minimal references

- **kuse_cowork** (Rust+Tauri, BYOK, 130 files) is a second same-stack reference after NeuralAgent, with `src-tauri/src/{agent,mcp,skills,tools,claude.rs,llm_client.rs,database.rs}` - a compact map of how a Tauri cowork agent lays out MCP, skills and tools modules. Useful as a sanity check on LiteCowork's crate boundaries; nothing to adopt wholesale.
- **Open-Claude-Cowork** (53 files, Electron) is a minimal Claude-Cowork alternative - a product-shape reference only.
- **nimbalyst** ships a `.claude-plugin/` packaging surface and an in-app update/notify flow (capability 17 updates); reference for the Feature 019 updater UX.
- **openwork** has an extension registry (`domains/settings/extension-registry.tsx`) and an `extensions-manifest-foundation.md` - another data point that the SKILL.md/.claude-plugin manifest convergence (Section 2) is the ecosystem norm.

## 6. open-design capability atoms: a catalog of capabilities to consider

open-design's daemon exposes capabilities as composable "atoms" (`apps/daemon/src/plugins/atoms/`): `code-import`, `design-extract`, `figma-extract`, `diff-review`, `diff-review-genui-bridge`, `patch-edit`, `rewrite-plan`, `token-map`, `build-test`, `handoff`, `auto-surfaces`. Several map to LiteCowork gaps worth considering as later capabilities: a `diff-review` atom bridged to generative UI (reinforces Feature 008 P-13 and 016), a `handoff` atom (reinforces Feature 015 delegation handoff), and `build-test` as a first-class verification step (reinforces the evidence model, capability 17). These are candidate capabilities, not V1 commitments; recorded for the roadmap.

## 7. What this changes for LiteCowork

Nothing here reopens an accepted decision. It strengthens the direction already set and adds concrete references:

| Finding | Reinforces / adds | LiteCowork home |
|---|---|---|
| Capability seam (Definition/Provider/Consumer) | The ACP adapter boundary and provider structure | A-54; 002 P-11; 005 Environment providers; 008 terminal/browser providers |
| Version-exemption + build-approval gating | Agent Upgrade Shield; plan-digest approval | 002 (upgrade shield), 013 |
| SKILL.md + .claude-plugin -> one manifest | LiteSPM/skills manifest normalization | 013 P-17 |
| Trust-tier-by-source + capability-grant + workspace-scoped visibility | "installed is not authorized"; Coworker/Workspace scope | 013, 009, 006 |
| MCP Apps interactive pages, reference-selection-to-chat, revisioned editable docs | Intelligent UI and co-edit | 016 P-03, 012, 007 |
| Worktree-per-task parallel agents + real browser extension host | Coding cockpit; managed browser | 005, 008 P-13, P-06 |
| Capability atoms (diff-review, handoff, build-test) | Roadmap capabilities | 008, 015, 017 roadmap |

## 8. Honest limits

Read read-only. No rival code was copied into LiteCowork; the clones stay in the gitignored `repo_take/`. Depth was architecture-level with exact-file citations (DeepSeek `plugin-manager/index.ts` and the capability-seam note; open-design `plugins/registry.ts`; ZCode `UI_PLUGIN.md`; paradev structure; the small repos near-whole), not an exhaustive line-by-line read of 1.9 million lines. Licenses must be re-checked at each repo's root before any pattern is adapted into code; this document adapts no code. The DeepSeek Harness "plugin-based architecture" the steering flagged is real and substantial (a profile-scoped plugin manager plus the capability-seam model), unlike `awesome-deepseek-harness`, which is only a catalog list.

## 9. Proposed tasks for owner review

Packages 002, 005, 008 are ACCEPTED/LOCKED (owner approval 2026-11-09 governs additions); 009, 013, 016 are IN_REVIEW.

| ID | Package | Proposed task |
|---|---|---|
| P-20 | 002 / 005 / 008 (cross-cutting) | Adopt the capability-seam structure explicitly for swappable providers: one Service Definition per seam (ACP agent, Environment, terminal, browser), sibling Service Providers, Consumers that never import provider types. Record it as an ADR. |
| P-21 | 002 | Agent Upgrade Shield: version-compatibility record per binding with an explicit owner-granted exemption (the DeepSeek `setVersionExemption(acceptRisk=true)` pattern) so a native agent upgrade never silently re-qualifies a binding. |
| P-22 | 013 | Normalize package manifests from `SKILL.md`, `.claude-plugin/plugin.json` and a LiteSPM sidecar into one validated manifest (the open-design `adapt*`/`mergeManifests`/`validateSafe` pattern), with trust-tier-by-source and capability grants derived from `(manifest, trust)`. |
| P-23 | 009 / 006 | Workspace/Coworker-scoped capability visibility with a fail-closed activation fence across async reads (the open-design `pluginVisibleFromWorkspace` + activation-fence pattern), so a grant is scoped and a tombstone is terminal. |
| P-24 | 016 / 012 | Build Intelligent UI and co-edit on MCP Apps (`ext-apps`): interactive pages via typed resource + `_meta.ui`, revisioned editable documents, "reference selection back to chat", and active-page preservation across Workbench tab switches (the ZCode UI Plugin model). |
| P-25 | 008 | Worktree-per-task parallel coding and a real managed-browser extension host (the paradev model) folded into Feature 008 P-13 scope and Feature 005 worktrees. |
| P-26 | roadmap (013/008/015/017) | Evaluate open-design capability atoms (diff-review-to-GenUI, handoff, build-test as verification) as post-V1 capabilities; record in the roadmap, not V1. |
