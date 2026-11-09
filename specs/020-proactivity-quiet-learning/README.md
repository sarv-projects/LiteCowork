# 020 - Proactivity and Quiet Learning

- **Status:** IN_REVIEW - authored complete for owner review on 2026-11-09; not owner-accepted; authorizes no implementation; not a claim of implementation or qualification.
- **Revision:** 0.1 | **Updated:** 2026-11-09 | **Blocker:** B-45 (first learning source and auto-commit eligibility policy), PROPOSED, pending owner. Design decision D-01 (first Suggestion producers) is accepted only with the package.
- **Owners:** Responsibility domain, Coworker, Storage, Operator/API, Desktop UI, Security/Trust, Product.
- **Release matrix rows:** Windows 11/10 x64 + OpenCode v2.0.32 first; Linux x64 + OpenCode v2.0.32 second.

## User outcome

The user sees a few explained ideas that never act on their own and respect quiet hours. A Coworker quietly remembers the user's explicit, low-risk private preferences with a subtle "Remembered" notice and Undo, sends anything riskier to a review list, and never learns from third-party or model content.

## Current truth

Suggestions, Goals and Needs You are SERVED with UI (`suggestions_operator.rs`, `goal_operator.rs`, `needs_you_operator.rs`, `suggestions_bridge.rs`, `src/suggestions/`), but nothing produces a Suggestion, so the Ideas page is empty in a real install. The learning gates in `crates/storage-core/src/coworker_learning.rs` are pure functions with no caller, storage, route or UI, and contain three defects this package fixes first (SPEC section 7.1). Feature 009 defers automatic learning (B-24) and names this package as the pipeline owner.

## Package

- [SPEC.md](SPEC.md) - problem, goals, D-01 and B-45, HLD/LLD, security, EARS acceptance.
- [PLAN.md](PLAN.md) - phases, dependency graph, rollout and gates.
- [TASKS.md](TASKS.md) - T01 to T17.
- [DATA-MODEL.md](DATA-MODEL.md) - Suggestion (existing) and learning records, lifecycles, retention, migration.
- [RESEARCH.md](RESEARCH.md) - sources, pinned references, alternatives.
- [WIRING.yaml](WIRING.yaml) - producers, services, stores, bridges and UI.
- [contracts/](contracts/README.md) - API/events, errors/recovery and UI behavior.

## Canonical owners

[Responsibilities](../../docs/RESPONSIBILITIES.md), [Learning](../../docs/LEARNING.md), [ADR-0016](../../docs/adr/0016-coworker-goal-and-suggestion-authority-boundary.md), [ADR-0023](../../docs/adr/0023-host-guidance-is-not-capability-authority.md), [ADR-0024](../../docs/adr/0024-optional-chat-first-coworkers.md), [Coworker flows](../../docs/COWORKER-FLOWS.md), [API](../../docs/API.md), [Events](../../docs/EVENTS.md), [Benchmarks](../../docs/BENCHMARKS.md), Features [009](../009-coworkers-scoped-memory/README.md) and [010](../010-automations-recurring-host/README.md).
