# 010 - Automations and Recurring Host

**Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.
**Revision:** 1.0 | **Updated:** 2026-11-09 | **Owner acceptance:** B-26, B-27 and the A-35 quiet-hours ownership accepted 2026-11-09 by the repository owner via [docs/OWNER-ACCEPTANCE-REVIEW-PACKET.md](../../docs/OWNER-ACCEPTANCE-REVIEW-PACKET.md).
**Implementation state (2026-11-10):** Not implemented beyond pre-existing partial foundations. T08, T09, T15 BLOCKED, T18 BLOCKED (not V1), T19 DEFERRED out of V1 by the owner; migration v44 allocated.
**Owners:** Automation, Task Runtime, Runtime lifecycle, Desktop UI.

## User outcome

A saved schedule or event creates exactly one Task per occurrence, including across sleep and restart, with truthful history and notification status.

## Current truth

The active checkout contains the Rust daemon, Tauri desktop application and shared crates. This package is ACCEPTED/LOCKED (owner-accepted 2026-11-09). That authorizes implementation in the order given by [TASKS.md](TASKS.md); it is not a claim that delivery, event triggers or Task execution exist. Its current implementation boundary is mapped in docs/IMPLEMENTATION-STATUS.md; do not infer qualification from a matching route, type or screen.

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


## Workflow boundary

This package owns Automations, occurrences and Routine-to-Task materialization; it does not own a native graph editor or workflow runtime. The target is to connect existing external workflows through a future qualified WorkflowBackendAdapter and invoke them as bounded CapabilityInvocations within Tasks. No production adapter is present in the active source tree, so selection/invocation is not currently a served LiteCowork feature. Users may separately use n8n Assistant/MCP or Activepieces Build by Asking/MCP inside those provider products, subject to provider availability and permissions. A LiteCowork Coworker cannot currently mutate the provider graph; direct Coworker graph-authoring tools are deferred to 010-T06 and require a future provider-specific authorization decision. See [CONNECTORS-WORKFLOWS](../../docs/CONNECTORS-WORKFLOWS.md), [SECURITY](../../docs/SECURITY.md) and the [OpenBot source study](../../docs/OPENBOT-REFERENCE-IMPLEMENTATION-STUDY.md).
