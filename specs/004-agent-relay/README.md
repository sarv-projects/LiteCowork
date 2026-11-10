# 004 - Agent Relay on One Conversation

**Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.
**Revision:** 1.0 | **Updated:** 2026-11-09
**Implementation state (2026-11-10):** Not implemented. Some tasks carry PARTIAL pre-existing foundations (see TASKS.md); no 2026-11-10 work.
**Owners:** Conversation, Agent Runtime, Product. Accepted by the repository owner via [docs/OWNER-ACCEPTANCE-REVIEW-PACKET.md](../../docs/OWNER-ACCEPTANCE-REVIEW-PACKET.md).

## User outcome

On one Conversation the owner runs a committed turn on agent A, a later turn on agent B that sees only portable context, then returns to A without copying private transcripts or credentials.

## Current truth

The active checkout contains the Rust daemon, Tauri desktop application and shared crates. B-15 and B-16 are owner-accepted (2026-11-09) and implementation is authorized. Today the supervisor has one dispatch arm (OpenCode), no Claude Code arm, and the desktop drops producer identity in `parseConversationSnapshot`; there is no selectable A→B→A pair (SPEC §4). The implementation boundary is mapped in docs/IMPLEMENTATION-STATUS.md; do not infer qualification from a matching route, type or screen.

## Lifecycle, scope and dependencies

- Lifecycle: ACCEPTED/LOCKED (2026-11-09). IMPLEMENTING when the first TASKS.md task starts; VERIFIED only through the final task with owner sign-off.
- Relay pair: agent A OpenCode v2.0.32, agent B Claude Code CLI. Release matrix: Windows x64 first, Linux x64 second.
- Depends on [Feature 001](../001-conversation-session-supervisor/SPEC.md) (turn admission, settlement, recovery), [Feature 002](../002-native-agent-install-auth-binding/SPEC.md) (OpenCode binding fix, descriptor, Claude Code module) and [Feature 003](../003-workspace-roots-context-ingestion/SPEC.md) (Resource materialization). Delivery wave 4 (gate E).

## Package

- [SPEC.md](SPEC.md)
- [PLAN.md](PLAN.md)
- [TASKS.md](TASKS.md)
- [DATA-MODEL.md](DATA-MODEL.md)
- [RESEARCH.md](RESEARCH.md)
- [WIRING.yaml](WIRING.yaml)
- [contracts/](contracts/README.md)

## Canonical owners

[Architecture](../../ARCHITECTURE.md), [Product](../../docs/PRODUCT.md), [V1 release](../../docs/V1-PRODUCT-RELEASE-CONTRACT.md), [Master plan](../../docs/IMPLEMENTATION-MASTER-PLAN.md), [Specification standard](../../docs/SPECIFICATION-STANDARD.md). Domain narrative: [RESEARCH.md](RESEARCH.md).
