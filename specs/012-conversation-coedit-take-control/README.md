# 012 - Conversation co-edit and take-control

**Status:** DRAFT - review required; does not authorize implementation.
**Revision:** 0.1 | **Updated:** 2026-11-08
**Owners to approve:** Conversation, Agent Runtime, Product.

## User outcome

While a native agent turn is running on an ordinary Conversation, the owner can step in on the same Conversation: take control of the live turn (request an observed stop and continue by hand) or open a bounded side-chat exchange that references the running turn, without the exchange becoming a second durable Task, Attempt or Coworker. Co-edit is a Conversation presentation and take-control UX, not a new durable work type.

## Current truth

The active checkout contains the Rust daemon, Tauri desktop application and shared crates. There is no co-edit or take-control surface today: this capability is tracked only as research gate RV-14 (Claude Duet co-edit), and the Conversation Stop control is itself ineffective for the one wired OpenCode path (register A-25). This package is DRAFT and does not authorize implementation. Its boundary is mapped in [docs/IMPLEMENTATION-STATUS.md](../../docs/IMPLEMENTATION-STATUS.md); do not infer qualification from a matching route, type or screen, and do not assert Claude Duet as a shipped fact.

## Package

- [SPEC.md](SPEC.md)
- [PLAN.md](PLAN.md)
- [TASKS.md](TASKS.md)
- [DATA-MODEL.md](DATA-MODEL.md)
- [RESEARCH.md](RESEARCH.md)
- [WIRING.yaml](WIRING.yaml)
- [contracts/](contracts/README.md)

## Canonical owners

[Architecture](../../ARCHITECTURE.md), [Product](../../docs/PRODUCT.md), [V1 release](../../docs/V1-PRODUCT-RELEASE-CONTRACT.md), [Master plan](../../docs/IMPLEMENTATION-MASTER-PLAN.md), [Specification standard](../../docs/SPECIFICATION-STANDARD.md). Conversation send authority is Feature 001; relay is Feature 004. Research gate: [RV-14](../../docs/VIABILITY-VS-COMPETITORS.md). Domain narrative: [RESEARCH.md](RESEARCH.md).
