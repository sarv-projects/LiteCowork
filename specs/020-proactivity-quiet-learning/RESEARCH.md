# Research and Decision Log - Proactivity and Quiet Learning

**Feature:** 020 | **Status:** IN_REVIEW - not owner-accepted; authorizes no implementation; not a claim of implementation or qualification.
**Checked:** 2026-11-09 - repository audit at HEAD 234c564 on `docs/audit-competitor-superset`. External references are the pinned revisions recorded in [REFERENCE-IMPLEMENTATIONS.md](../../docs/REFERENCE-IMPLEMENTATIONS.md); they were not re-cloned for this package.

## 1. Repository sources inspected

| Source | Path | Finding | Limit |
|---|---|---|---|
| Suggestion routes | `apps/litecoworkd/src/suggestions_operator.rs` (1,417 lines) | list (with expiry settlement), resolve, accept-task, snooze, preferences; ARCHIVED Coworker rejection at line 728 | No propose path |
| Suggestion domain | `crates/domain-responsibility/src/suggestions.rs` | Types, `SuggestionService::{settle_expired, owner_action, kind_preferences, set_kind_preference}`; header "proposal production remains separate" | No producer |
| Suggestion storage | `crates/storage-sqlite/src/suggestions.rs` | Expiry, owner action, preference, atomic Task acceptance and receipt | No insert path outside tests |
| Schema | `docs/schemas/sqlite-v1.sql` | `suggestions`, `uq_suggestions_open_dedupe`, `idx_suggestions_dismissal_cooldown`, `suggestion_preferences`, `notification_preferences.quiet_hours_json` | - |
| Events | `docs/schemas/domain-event.schema.json`, `docs/EVENTS.md` | suggestion.* and goal.* events defined; no learning.* | - |
| Bridge/UI | `apps/litecowork-ui/src-tauri/src/suggestions_bridge.rs` (338 lines), `apps/litecowork-ui/src/suggestions/` (879 lines) | Validated commands, Ideas page with recovery and live region | No quiet-hours behavior |
| Goals | `apps/litecoworkd/src/goal_operator.rs` (533 lines), `crates/domain-responsibility/src/goals.rs` | Passive revisioned Goals; progress from verified outcomes | No conformance tests for "never schedules" |
| Needs You | `apps/litecoworkd/src/needs_you_operator.rs` (211 lines), `crates/storage-core/src/needs_you.rs` | Read-only, no-store | - |
| Learning gates | `crates/storage-core/src/coworker_learning.rs` (1,711 lines, 30 tests) | Deterministic gates; no caller | Three defects (SPEC 7.1) |
| Quiet hours | `crates/domain-responsibility/src/notifications.rs` | `QuietHours::{parse, contains}`; event classes for Automation/Task/Dependency only | Editor not built (A-28) |
| Owner docs | `docs/LEARNING.md`, `docs/RESPONSIBILITIES.md`, ADR-0016, ADR-0023, ADR-0024, `docs/COWORKER-FLOWS.md` CF14, `docs/BENCHMARKS.md` B18, B54, B55, B61, B62, B65, B72 | Target contracts and benchmarks | LEARNING vocabulary note still says PROPOSED although A-20/A-34 was accepted 2026-11-09 (T02 updates it) |

## 2. Pinned external references

| Ref | Source and revision | License | Use | Decision |
|---|---|---|---|---|
| Agenta | [Agenta-AI/agenta](https://github.com/Agenta-AI/agenta/blob/4bc51903a1597d3ef8f1ac17f2d6f472ba7c8137/README.md) at `4bc51903a1597d3ef8f1ac17f2d6f472ba7c8137` | per upstream LICENSE (not re-checked here) | Separation of memory, instruction, skill and automation; opt-in self-configuration tools | **Adopt the separation** (already LOCKED in LEARNING); **reject** model-driven direct config mutation |
| REF-ROWBOAT | [rowboatlabs/rowboat](https://github.com/rowboatlabs/rowboat/tree/9052e28b9ede46478e04f9cf600c7ebaf5895dc7) at `9052e28b9ede46478e04f9cf600c7ebaf5895dc7` | Apache-2.0 | Desktop coworker that indexes work into a knowledge graph | **Study** only; reject automatic indexing of every source as memory (ADR-0024 rejected alternative) |
| REF-OPENWORK | [different-ai/openwork](https://github.com/different-ai/openwork/tree/5493b18a6af64d56454658a53da00eabbc6e033d) at `5493b18a6af64d56454658a53da00eabbc6e033d` | MIT except `/ee` | Automation receipts stay readable after the admitting process stops | **Study** for producer cursor durability; do not import `/ee` |

## 3. Alternatives considered

| Topic | Options | Choice (PROPOSED) | Reason |
|---|---|---|---|
| First Suggestion producers | Model-generated ideas; deterministic rules over committed state | Deterministic GOAL_NEXT_STEP and REPEATED_TASK (D-01) | RESPONSIBILITIES allows deterministic producers; no quota use; reproducible tests |
| Quiet hours | Suppress creation; suppress push only | Suppress push surfaces only | The owner can still pull ideas; nothing is lost; matches CF14 "Needs You stays discoverable" |
| First learning source | Owner messages; Task outcomes; demonstrations; imported docs | Owner messages and owner feedback in Coworker Conversations (B-45) | LEARNING section 16 default eligible sources; Task outcome excluded by default policy |
| Candidate creation | Model extractor only; deterministic owner action only; both | Owner action first, optional extractor second | Works without a qualified classifier; extractor stays untrusted |
| CONFIRMED certainty | Trust extractor label; verbatim span plus lexicon | Verbatim span plus lexicon | Model output cannot raise its own certainty |
| Inferred auto mode | Offer in V1; defer | Defer | Narrowest safe V1; review path covers inferred items |
| Gate module location | storage-core; domain-responsibility | Decide in T03 (F-04) | One definition; no new crate |

## 4. Unresolved questions

See SPEC section 17 (O-01 to O-06).
