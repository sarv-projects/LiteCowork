# OpenDesign (nexu-io/open-design) capability deep-dive (2026-11-09)

**Status:** research study. Not an owner contract. Authority stays with ARCHITECTURE.md, AGENTS.md and the accepted packages. Lessons map to proposed tasks (Section 8); no accepted package code is changed here.
**Source:** [nexu-io/open-design](https://github.com/nexu-io/open-design) at HEAD `07a86b1`, Apache-2.0, cloned read-only into `repo_take/open-design` (gitignored, never imported, per AGENTS.md section 7).
**Depth:** line-level read of the capability-critical daemon modules. Read in full: `apps/daemon/src/plugins/{trust.ts, registry.ts, apply.ts (head), installer.ts (structure)}`, `packages/contracts/src/plugins/manifest.ts`, and the module maps of `apps/daemon/src/{agent-protocol/acp, genui, live-artifacts, collab, plugins/atoms}`. This is the repo you flagged as the most important for adding capabilities; it is also Apache-2.0, so its patterns are the most freely adaptable.

## 1. What OpenDesign is, and why it matters to LiteCowork

OpenDesign is a collaborative AI design-agent workspace with a Rust-free TypeScript daemon (`apps/daemon`) that is strikingly close to LiteCowork's intended shape: a plugin subsystem (14,527 lines), an **ACP agent-protocol** client, a **GenUI** registry, **live-artifacts** (dashboards that refresh from connected data), a deep **collab** layer, strategies/atoms as composable capabilities, and a trust/capability-grant model. It is the richest single source in the whole rival set for capabilities 7 (marketplace/plugins), 8 (connectors), 13 (Intelligent UI and interactive artifacts) and 14 (dashboards). Three of its subsystems are directly borrowable patterns for accepted LiteCowork packages.

## 2. The capability-grant model (plugins/trust.ts) - the best reference for LiteCowork Trust

This is production-grade and maps almost one-to-one onto LiteCowork's Trust/grant needs (006/009/013):

- **Two trust tiers, source-defaulted.** `defaultTrustForRecord` returns `trusted` only for a `local` install (the user copied it); bundled/marketplace/github/url/project default to `restricted` until an explicit flip (`trust.ts:~30`). This is LiteCowork's "installed is not authorized" rule made concrete.
- **A closed capability vocabulary.** `KNOWN_TOP_LEVEL_CAPABILITIES` = `prompt:inject, fs:read, fs:write, mcp, subprocess, bash, network, connector, pipeline:*, genui:custom-component` (`trust.ts:~110`). Scoped grants `connector:<id>` and `mcp:<name>` are matched by regex (`SCOPED_CONNECTOR_RE`, `SCOPED_MCP_RE`). Anything else is rejected at the HTTP layer, not silently dropped - the comment is explicit: "unknown shapes are NOT silently dropped - the caller must surface them" (`validateCapabilityList`, `trust.ts:~140`). LiteCowork's grant surface should have exactly this closed vocabulary with scoped forms and loud rejection.
- **Required vs granted are computed separately.** `requiredCapabilities(manifest)` walks the manifest's `mcp`, `connectors.required/optional` (optional carries a `?` suffix), `genui.surfaces`, `pipeline.stages` and explicit `capabilities` into the set a plugin needs to apply; `resolveCapabilitiesGranted({manifest, trust})` grants the trusted defaults plus required caps for a trusted plugin, or only `prompt:inject` for a restricted one (`trust.ts:~40-90`). A doctor reports under-grants by diffing the two. LiteCowork's Discover should show "this package requires X, you have granted Y."
- **A floor that revoke cannot breach.** `revokeCapabilities` always preserves `prompt:inject` and refuses to drop it (`trust.ts:~190`). Grants are an idempotent sorted union (`grantCapabilities`). This is the clean shape for LiteCowork capability grants: additive, idempotent, with an inviolable minimum.
- **The GenUI escape-hatch is gated.** A restricted plugin cannot "smuggle arbitrary UI through the GenUI layer"; a bundled React component surface requires the explicit `genui:custom-component` capability (`trust.ts:~120` comment). This is precisely LiteCowork's capability-13 rule that rich UI uses registered components and never arbitrary model markup - here enforced as a named capability.

## 3. The manifest model (contracts/plugins/manifest.ts) - adopt, do not invent

OpenDesign's `open-design.json` is a Zod schema (`manifest.ts`) that is deliberately permissive at the top level so that **adapter outputs** from `SKILL.md` frontmatter and Claude `plugin.json` parse into the same `PluginManifest` without losing fields. It defines `McpServerSpec` (name, command, args, env, url - stdio or remote), `InputField` (string/text/select/number/boolean/file, required, options, i18n labels), `LocalizedText` with a locale-fallback resolver, and `PipelineStage` (id, atoms, repeat, until, onFailure: abort/skip/retry). LiteCowork's LiteSPM/skills manifest work (013, P-22) should take this wholesale as a design: accept SKILL.md and .claude-plugin natively, normalize to one validated manifest, carry MCP specs and typed input fields, and localize display text with a fallback chain. The input-field types are also a ready vocabulary for the Intelligent-UI form surfaces (capability 13).

## 4. The apply pipeline (plugins/apply.ts) - pure, probe-aware, provenance-stamped

`applyPlugin` is a **pure** function (no SQLite, no FS, no network - side effects belong to the caller; `apply.ts:~15` header). It:
- validates inputs against `manifest.od.inputs`, raising `MissingInputError` surfaced as HTTP 422;
- resolves connector bindings against a live `connectorProbe` and **auto-derives an `oauth-prompt` GenUI surface for any required-but-not-connected connector** (`apply.ts:~95`, `connector-gate.ts deriveAutoOAuthPrompts`) - the "Needs You: connect this account" flow (capability 15) generated from the manifest;
- coerces the trust tier to a binary `trusted|restricted` so the snapshot's permission contract is unambiguous;
- resolves localized strings **at apply time and stores them in the snapshot** so historical runs never change when translations do (`apply.ts:~80`) - a provenance discipline LiteCowork's evidence model (capability 17) should copy;
- produces an `ApplyResult` that is the single contract between apply and project-create, the run's system-prompt block, and the context chip strip.

The purity split (compute vs persist) is exactly how LiteCowork's accepted packages are supposed to be structured and a good template for Feature 013's plan/apply and Feature 001's context assembly.

## 5. A second ACP client (agent-protocol/acp) - corroborates A-54

OpenDesign ships its own ACP client under `apps/daemon/src/agent-protocol/acp/`: `session.ts`, `rpc.ts`, `stdio-mcp.ts`, `tool-execution-lifecycle.ts`, `session-params.ts`, `emission-provenance.ts`, `json-line-stream.ts`, with `ACP_PROTOCOL_VERSION = 1`, a per-stage inactivity watchdog that **resets on every received line** (bounding silent periods, not total runtime, default 10 min, env-overridable, `<=0` disables), and artifact-detection patterns that recognize an agent emitting a file. This is the second independent production ACP client in the rival set (after paseo), which strongly corroborates the LiteCowork decision to standardize the native-agent boundary on ACP (register A-54, P-11). Two specific patterns to borrow: the **line-resetting inactivity watchdog** (better than DeepSeek's and far better than LiteCowork's current hard 60 s cap, A-52/R-02), and **emission-provenance** (tagging where each streamed emission came from), which feeds the honest REPORTED/OBSERVED model.

## 6. Live artifacts and GenUI - capabilities 13 and 14

- **live-artifacts** (`refresh-service.ts`, `refresh.ts`, `render.ts`, `schema.ts`, `store.ts`) implements dashboards that re-pull current data when reopened - the "live artifact" capability the feature list and the competitive audit call out (capability 13 layer 4, capability 14 Workspace Dashboard). It is schema-validated and rendered through a registry, not free HTML. LiteCowork's Feature 016 (Intelligent UI) and the proposed Workspace Dashboard (P-16) should model live widgets the same way: a typed schema, a refresh service bound to a Connection, host-rendered components.
- **genui** (`registry.ts`, `store.ts`, `events.ts`) is a registered-component GenUI layer with an event store - the same registered-renderer discipline LiteCowork's 016 requires, confirming the design.

## 7. Capability atoms and the collab layer

- **atoms** (`plugins/atoms/`: `code-import`, `design-extract`, `figma-extract`, `diff-review`, `diff-review-genui-bridge`, `patch-edit`, `rewrite-plan`, `token-map`, `handoff`, `build-test`, `auto-surfaces`) are composable capability units a pipeline stage lists by id. For LiteCowork these are a menu of post-V1 capabilities: `diff-review-genui-bridge` (a diff rendered as interactive GenUI) reinforces Feature 008 P-13 and 016; `handoff` reinforces Feature 015 delegation; `build-test` as a first-class stage reinforces the evidence model.
- **collab** (113 files: comment-file authority/identity-recovery/publication-coordination, workspace-selection, authorized-team-project-pull, cancel-revoked-workspace-runs) is a mature multi-user collaboration layer. It is explicitly V2 for LiteCowork (multi-user/shared Workspaces), so it is a reference to revisit then, not now - but its "cancel runs when a workspace grant is revoked" and "comment-file identity recovery" patterns are worth remembering for the fencing discipline.

## 8. What LiteCowork should take, mapped to packages

| OpenDesign pattern | Evidence | LiteCowork home |
|---|---|---|
| Two-tier trust defaulted by source; closed capability vocabulary with scoped `mcp:`/`connector:` forms; loud rejection of unknown shapes | `plugins/trust.ts` | 006/009/013 grant surface (P-23 strengthened) |
| required-vs-granted capability diff; `prompt:inject` floor revoke cannot breach; idempotent sorted-union grants | `plugins/trust.ts` | 013 Discover under-grant reporting; 009 Coworker grants |
| One Zod manifest fed by SKILL.md + .claude-plugin adapters; MCP specs, typed input fields, localized text, pipeline stages | `contracts/plugins/manifest.ts` | 013 manifest normalization (P-22) |
| Pure apply (compute vs persist); auto-derived OAuth prompt for unconnected required connector; apply-time localized-string snapshotting | `plugins/apply.ts`, `connector-gate.ts` | 013 plan/apply; 006 connector onboarding; 015 Needs You |
| Second production ACP client; line-resetting inactivity watchdog; emission provenance | `agent-protocol/acp/` | A-54/P-11 corroborated; 001-T04 watchdog replaces the hard 60 s cap (A-52/R-02); evidence model |
| Live artifacts: schema-validated, registry-rendered, refresh-from-Connection dashboards | `live-artifacts/`, `genui/` | 016 Intelligent UI; P-16 Workspace Dashboard |
| Capability atoms (diff-to-GenUI, handoff, build-test) | `plugins/atoms/` | roadmap (008/015/017), P-26 |

Proposed task updates (owner review): **P-23** is strengthened to adopt the open-design trust/grant model (closed vocabulary, scoped forms, required-vs-granted, inviolable floor). New **P-28**: replace Feature 001's hard 60 s turn deadline with a line-resetting inactivity watchdog (open-design and the ACP model), feeding 001-T04. **P-22** is confirmed to take the open-design manifest-adapter design. **P-16** (Workspace Dashboard) adopts the live-artifacts schema/refresh model.

## 9. Honest limits

Line-level read of the capability-critical modules (trust, registry, apply, manifest, and the acp/genui/live-artifacts/collab/atoms module maps), cited by construct. I did not read all 14,266 files (most are the web app, brands, design-systems, media adapters, localized docs and tests). No OpenDesign code was copied into LiteCowork; the clone stays gitignored. Apache-2.0 at the root is the most permissive license in the rival set, but any adaptation into code must still re-check the exact file; this document adapts no code.
