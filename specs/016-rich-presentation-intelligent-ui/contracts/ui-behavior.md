# UI behavior - Rich answers, chart, calculator, media and interactive preview (Feature 016)

**Status:** IN_REVIEW - authored for owner review; binds UI to owner contracts; not pixels.
**Normative UI indexes:** [V1-UI-SCREEN-CONTRACT.md](../../../docs/V1-UI-SCREEN-CONTRACT.md), [V1-UI-ACTION-STATE-LEDGER.md](../../../docs/V1-UI-ACTION-STATE-LEDGER.md), [DESKTOP-SHELL-LLD.md](../../../docs/DESKTOP-SHELL-LLD.md).
**Wire:** [LOCAL-OPERATOR-IPC.md](../../../docs/LOCAL-OPERATOR-IPC.md) and [operator-api.openapi.yaml](../../../docs/schemas/operator-api.openapi.yaml).

The UI renders validated documents and semantic text. Presentation elements never call host commands.

## 1. User flow and transitions

| Step | Screen / surface | Transition |
|---|---|---|
| Read answer | Conversation message (`RichResponseView` in `ConversationsPage.tsx`) | Semantic text first; rich view replaces it when a valid document loads |
| Read original | "Read the original response" disclosure | Expands the full semantic text |
| See data | Chart > "Show data table" toggle | Toggles the equivalent table under the chart |
| What-if | Calculator inputs | Result updates live; "Reset to original values" restores |
| Turn off rich view | Settings > display preferences > "Show formatted answers" switch | All messages render semantic text |
| Open image | Message Artifact chip or Library item > Artifact view | Image with alt text, zoom and download (Feature 007 controls) |
| Open interactive app | Library or Workbench item | Inert preview with "Running interactive apps is not available in this version." |

## 2. UI states

| Surface | States | Meaning |
|---|---|---|
| Message | semantic-only, loading-rich (semantic shown), rich, fallback (semantic shown, no error banner), disabled | The answer is always readable |
| Chart | chart, table-open, fallback-table | Fallback when binding or values fail |
| Calculator | initial, edited, cannot-calculate | Fallback text always visible |
| Image Artifact | loading, shown, rejected-at-ingest (no Artifact exists), error | Error offers retry of the read only |
| Interactive Artifact | preview-only | No Run control |

## 3. Validation and failure presentation

| Validation | Rule | UI state |
|---|---|---|
| Document invalid on the desktop | `parseRichPresentationResponse` returns null | semantic-only, no error banner |
| Unknown renderer contract version | not 1 or 2 | semantic-only |
| Calculator input | finite number, else the field shows "Enter a number" | cannot-calculate until fixed |

## 4. Commands (UI to core)

| UI control | Local action | Command |
|---|---|---|
| Message render | fetch presentation if enabled | `get_rich_presentation` (existing) |
| Show formatted answers switch | set preference | Feature 011 display preference command |
| Chart table toggle | view-local | none |
| Calculator inputs and Reset | view-local reducer | none |
| Open image | navigate | Feature 007 Artifact read |

## 5. Events and stream (core to UI)

| Source | Type | UI use |
|---|---|---|
| Operator stream | `projection` frames for the Conversation | New presentation reference for a message; resync on `stream.resync_required` |
| Read APIs | Conversation presentation, rich presentation envelope | Hydrate on open |

## 6. Accessibility

Rich view keeps the existing `<section aria-label="Assistant response">`. Tables use `<th scope="col">` and a focusable scroll region. Chart is `role="img"` with the accessible summary as its name; the data table toggle is a `<button aria-expanded>`. Calculator inputs have visible labels; the result sits in `aria-live="polite"`. Contrast at least 4.5:1; no animation when `prefers-reduced-motion` is set; colour is never the only signal in charts (pattern or direct labels). Target: no serious or critical axe violations.

## 7. Tests

AC-002, AC-004, AC-005, AC-006, AC-008, AC-009, AC-010, AC-013, AC-018 through `apps/litecowork-ui/tests/` (`rich-presentation-corpus.test.ts`, `chart-block.test.ts`, `safe-calculator-block.test.ts`, `interactive-artifact-preview.test.ts`, `presentation-no-exec-gate.test.mjs`, `rich-response-view-policy.test.ts`), run by `pnpm --dir apps/litecowork-ui test`.

## 8. Cross-layer trace (filled by TASKS T15)

| Operation | Control | Command | Route | Owner | Record | Recovery | Evidence |
|---|---|---|---|---|---|---|---|
| View rich answer | message | `get_rich_presentation` | `GET /v1/rich-presentations/{presentation_id}` | RichPresentationStore | `rich_presentations` | stream resync | to be recorded |
| Disable rich view | Settings switch | Feature 011 preference | Feature 011 | Feature 011 preference store | local preference | persisted across restart | to be recorded |
| Calculator | inputs | none (view-local) | none | view | none | none | to be recorded |
