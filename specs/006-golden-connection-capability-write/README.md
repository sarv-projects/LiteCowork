# 006 - Golden Connection and One Capability Write

**Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.
**Revision:** 1.0 | **Updated:** 2026-11-09
**Implementation state (2026-11-10):** Not implemented. T02 BLOCKED on the B-19 follow-up (GitHub OAuth client-secret finding, open decision), T05 BLOCKED behind it; migration v40 allocated.
**Owners:** Connectors, Capability Fabric, Security, LiteSPM boundary, Desktop UI. Accepted by the repository owner via [docs/OWNER-ACCEPTANCE-REVIEW-PACKET.md](../../docs/OWNER-ACCEPTANCE-REVIEW-PACKET.md).

## User outcome

The owner completes one real account Connection, performs one authorized read, then one consequential write with approval, idempotency and reconciliation, and revoke fences later use.

## Current truth

The active checkout serves Discover projections, fail-closed connect-or-install/test routes, exact Approval list/get/resolve and a narrow Task-planning Resource-read grant. Connect-or-install returns 503 without a receipt or grant; test returns UNKNOWN without contacting a provider. Account Connection/OAuth routes are not mounted; CapabilityInvocation is read-only; Trust/Effect provider dispatch and reconciliation are not wired. The Approval UI records an exact action decision but does not execute it. This package is ACCEPTED/LOCKED (B-19 and B-20 accepted 2026-11-09); no golden provider read/write journey is implemented or qualified. A 2026-11-09 provider check found that GitHub requires a client secret for the web-flow exchange and token revocation, which triggers B-19's own stop clause; task T02 in [TASKS.md](TASKS.md) holds the owner decision.

## Boundary

006 does not own the LiteSPM package lifecycle (observe, adopt, install, update, repair, unmanage, remove, inventory, Discover Managed/Installed tabs, E2E-06). That is package 013, litespm-package-lifecycle-bridge. 006 owns the account Connection, scoped read and write, approval, UNKNOWN reconciliation and revoke.

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
