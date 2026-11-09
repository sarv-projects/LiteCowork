# Research and Decision Log - Rich Presentation, Intelligent UI and Generated Media

**Feature:** 016 | **Status:** IN_REVIEW - authored for owner review; does not authorize implementation.
**Checked:** 2026-11-09, source audit of branch `docs/audit-competitor-superset`; upstream pin taken from the reference index, not re-fetched.

## 1. Repository sources inspected

| Source | Path | Finding |
|---|---|---|
| Desktop validator | `apps/litecowork-ui/src/presentation/rich-presentation.ts` | 512 KiB, 200 blocks, depth 8, 30,000 values, JSON depth 32; five kinds; RFC 8785 and SHA-256 checks; null on any failure |
| Renderer | `apps/litecowork-ui/src/presentation/RichResponseView.tsx` | Five kinds; table region with `tabIndex=0`; semantic disclosure |
| Rust validator | `crates/storage-sqlite/src/rich_presentations.rs` | Two kinds; closed objects; canonical bytes; provenance rules |
| Store contract | `crates/storage-core/src/rich_presentation.rs` | 1 MiB cap; stale DIVIDER comment; page limits |
| Operator | `apps/litecoworkd/src/presentation_operator.rs` | Read routes and stream; publish used only in tests |
| Schemas | `docs/schemas/rich-presentation.schema.json`, `presentation-intent.schema.json`, `interactive-artifact-runtime.schema.json` | 13 kinds in v1 schema; intent strategies; runtime IPC schema |
| Owner docs | `docs/RICH-RESPONSE.md`, `PRESENTATION-RUNTIME.md`, `INTELLIGENT-UI-AND-IMAGE-CONTENT.md`, `INTERACTIVE-ARTIFACT-RUNTIME.md`, `RENDERING-CONFORMANCE-LLD.md` | Display-only v1; CalcAst; chart table fallback; image pipeline; PREVIEW_ONLY |
| Plans | `docs/IMPLEMENTATION-MASTER-PLAN.md` P10, `docs/LITECOWORK-BUILD-TASKS.md` BT-06.3 to BT-06.5, `docs/ACCEPTANCE-AND-E2E-PLAN.md` E2E-11 | Exit: semantic answer complete when rendering disabled; malicious presentation cannot mint host state |
| E08-S09 | `docs/archive/legacy-implementation-c7e9c05/epics/E08.md` (historical) and `docs/BENCHMARKS.md` | "Isolated interactive Artifact Runtime" story that owns Run qualification |
| CSP | `apps/litecowork-ui/src-tauri/tauri.conf.json` | `script-src 'self'`; `img-src 'self' asset: http://asset.localhost data:` |
| Dependencies | `Cargo.lock`, `apps/litecowork-ui/package.json` | No jsonschema crate; no chart library; React 19 only |

## 2. Pinned external reference

| Reference | Pin | License | Use |
|---|---|---|---|
| REF-OPENINTELLIGENTUI `CopilotKit/OpenIntelligentUI` | `f6e4388b26a64b9a0714943b08a1ce622b924eec` | MIT at checked root LICENSE | UX, sandbox lifecycle, explicit bridge and renderer-test ideas only |

Source rows: [docs/REFERENCE-IMPLEMENTATIONS.md](../../docs/REFERENCE-IMPLEMENTATIONS.md) (REF-OPENINTELLIGENTUI) and [docs/SOURCE-RECONCILIATION.md](../../docs/SOURCE-RECONCILIATION.md) "OpenIntelligentUI review for E08-S08". No `repo_take/` clone exists.

### Copy / prototype / reject for this feature

| Mechanism | Verdict | How it applies here |
|---|---|---|
| Renderer behaviour tests (`renderer.tsx`) | copy test ideas only | Hostile-input and fallback cases in the shared corpus; no source reuse |
| Schema shapes (`schema.ts`) | test-shape reference only | Cross-check corpus coverage; LiteCowork schema remains canonical |
| Host-action and failure tests (`sandbox-functions.ts`, `prompt-bridge.tsx`) | reference | Bridge-call spy test proving no rendered element changes host state |
| Standalone prototype speed | prototype (outside the product) | Allowed only for throwaway UX exploration; never shipped |
| Arbitrary generated HTML/CSS/JavaScript renderer | reject | Violates L-03 and AGENTS.md section 8 |
| `eval`, `unsafe-eval`, `unsafe-inline` scripts, CDN origins in its CSP | reject | CSP stays `script-src 'self'` |

## 3. Alternatives considered

| Topic | Option | Decision |
|---|---|---|
| Validator authority | Keep validation in `storage-sqlite` | Rejected: a pure crate is reusable and testable; storage calls it |
| Schema engine | Add the `jsonschema` crate | Rejected for V1: hand-written closed objects are the repo pattern and keep errors typed; revisit with RESEARCH justification |
| First Intelligent UI component | TABS, ACCORDION, DATA_TABLE, SAFE_CALCULATOR | SAFE_CALCULATOR proposed (B-40): it is the only one that exercises the restricted AST and reducer BT-06.4 asks for |
| Chart rendering | Chart library, Mermaid, model SVG, host React SVG | Host React SVG: no dependency, no model markup |
| Separate IntelligentUiDocument | Second document per message | Rejected: one validated RichPresentation per message with renderer contract version 2 keeps one digest binding and one fallback path |
| Media in presentation blocks | MEDIA block in V1 | Deferred: images are Artifacts shown in the Artifact view; MEDIA block can follow once the display scheme is settled |
| Interactive Artifact Run | Enable behind a flag | Rejected: release contract requires E08-S09 evidence first |

## 4. Optimization notes

- The TypeScript validator computes one SHA-256 per TEXT_SLICE; for 200 slices this is the main cost. If AC-016 fails, hash the encoded document once and verify slice ranges against that buffer instead of re-encoding.
- Move validation to publish time in Rust so the desktop's work is a confirmation pass on already-valid documents; rejected documents never cost render time.
- Keep chart geometry and the calculator reducer as pure TypeScript modules so they run under `node --test` without a DOM.
- Do not split rendering into a Web Worker unless the 100 ms long-task budget fails on the release-row machine.

## 5. Open research and decision register

| ID | Question | Status |
|---|---|---|
| R-01 | Does OpenCode v2.0.32 emit structured presentation intent? | Open; binder derives from Markdown first |
| R-02 | Image display scheme without loosening CSP | Open; Feature 011 |
| R-03 | EXIF stripping for display copies | Open; Security |
| B-40 | First component and Interactive Artifact Runtime posture | PROPOSED, pending owner |
