# 016 - Rich Presentation, Intelligent UI and Generated Media

- **Status:** IN_REVIEW - authored complete for owner review; does not authorize implementation until the owner accepts the SPEC and blocker B-40.
- **Revision:** 0.1 | **Updated:** 2026-11-09 | **Decision:** B-40 (first Intelligent UI component and Interactive Artifact Runtime V1 posture) ACCEPTED as proposed 2026-11-10 (A-62); whole-SPEC acceptance pending, package stays IN_REVIEW.
- **Implementation state (2026-11-10):** Not implemented beyond pre-existing partial foundations; plans no migration.
- **Owners:** Presentation Runtime, Conversation, Artifacts, Storage, Operator/API, Desktop UI, Security, Accessibility and Product.
- **Release rows:** Windows x64 with OpenCode v2.0.32 first, Linux x64 second. No Environment provider is needed.

## User outcome

Answers can show tables, a chart with an equivalent data table, and one accessible "what-if" calculator built from the answer's own numbers. Turning rich presentation off, or any validation failure, shows the full original text. Generated images become Library Artifacts with their source Conversation and turn. Interactive app Artifacts open as an inert preview; nothing runs in the chat WebView.

## Current truth

The desktop validator (`apps/litecowork-ui/src/presentation/rich-presentation.ts`, 512 KiB, 200 blocks, depth 8) and the Rust storage validator (`crates/storage-sqlite/src/rich_presentations.rs`) accept the same five V1 block kinds (TEXT_SLICE, LAYOUT, CARD, CALLOUT, TABLE) under the same 512 KiB cap, and the served wire schema carries that cap too. Nothing in production publishes a RichPresentation. There is no chart, image or Intelligent UI renderer, no media ingestion, no disable preference and no Interactive Artifact Runtime code. The read route and the semantic fallback view exist.

Under this package's IN_REVIEW status, proposed task P-03 is folded in as 016-T18: per-block Markdown degradation with GFM tables, horizontal rules and nested lists; code blocks with a language label, copy button and lazy highlighting; TEXT_SLICE rendered through the same renderer; and a golden corpus of real agent answers that renders with no whole-message raw fallback and no HTML strings. Source: [docs/UI-RENDERING-SETTINGS-ARCHITECTURE-AUDIT-2026-11.md](../../docs/UI-RENDERING-SETTINGS-ARCHITECTURE-AUDIT-2026-11.md) section 9.

## Package

- [SPEC.md](SPEC.md) - problem, decisions (B-40), HLD/LLD, security, EARS acceptance.
- [PLAN.md](PLAN.md) - phases, dependency graph, gates, risks.
- [TASKS.md](TASKS.md) - ordered tasks to VERIFIED for V1.
- [DATA-MODEL.md](DATA-MODEL.md) - records, lifecycles, bounds.
- [RESEARCH.md](RESEARCH.md) - pinned references (OpenIntelligentUI) and decisions.
- [WIRING.yaml](WIRING.yaml) - components, edges, transactions, boundaries.
- [contracts/](contracts/README.md) - API/events, errors/recovery, UI behavior.

## Dependencies

Feature 001 (committed semantic message), Feature 007 (Artifact create and Conversation association), Feature 011 (CSP, display preferences, IPC), Feature 019 (CSP audit, release rows).

## Canonical owners

[Architecture](../../ARCHITECTURE.md), [V1 release](../../docs/V1-PRODUCT-RELEASE-CONTRACT.md), [Rich Response](../../docs/RICH-RESPONSE.md), [Presentation Runtime](../../docs/PRESENTATION-RUNTIME.md), [Intelligent UI and image content](../../docs/INTELLIGENT-UI-AND-IMAGE-CONTENT.md), [Interactive Artifact Runtime](../../docs/INTERACTIVE-ARTIFACT-RUNTIME.md), [Rendering conformance](../../docs/RENDERING-CONFORMANCE-LLD.md), [Artifacts and evidence](../../docs/ARTIFACTS-EVIDENCE.md), [Schemas](../../docs/SCHEMAS.md).
