# 008 - Workbench Browser Terminal and Computer

**Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.
**Revision:** 1.0 | **Updated:** 2026-11-09 | **Owner acceptance:** repository owner, 2026-11-09, via [OWNER-ACCEPTANCE-REVIEW-PACKET.md](../../docs/OWNER-ACCEPTANCE-REVIEW-PACKET.md)
**Implementation state (2026-11-10):** Not implemented. T09 BLOCKED on a Feature 005 Linux Environment, T12 BLOCKED on B-35 (Windows); migration v41 allocated.
**Component owners:** Desktop shell, Environments, Browser provider, Trust, Desktop UI.

## User outcome

The owner uses the right Workbench to finish real browser and terminal work, takes over and returns control, and a stale handle cannot act.

## Current truth

The active Workbench Files pane is a Resource projection/preview, not a live filesystem. Browser is an empty-by-default embedded iframe preview labelled as such, not Chrome or a browser-control provider. After register A-51 the Terminal pane (`TerminalPane`) and Review pane (`ReviewPane`) in `apps/litecowork-ui/src/workbench/WorkbenchPanel.tsx` render honest unavailable notices with no fabricated output, exit codes or containment labels. Activity has no rendered branch (the `activity` tab value falls through to Review); Computer capture/input is unavailable and has no status surface. No BrowserSession, PTY, Activity, Diff or Computer route/provider exists.

Accepted decisions (2026-11-09): B-22 `MANAGED_CHROMIUM` with an isolated profile, ambient-CDP discovery and default-profile copy disabled; B-23 one daemon-owned PTY/ConPTY per tab under an admitted Environment with ordered receipts and timeout-as-receipt, qualifying on the Linux x64 row first and visibly UNAVAILABLE on Windows until Feature 005 B-35; B-32 Computer use deferred and explicitly UNAVAILABLE. Revision 1.0 adds the Workbench Activity view and exact-revision/VCS Diffs to scope. The V1 plan is in [TASKS.md](TASKS.md).

By owner approval on 2026-11-09 (see [docs/OWNER-ACCEPTANCE-REVIEW-PACKET.md](../../docs/OWNER-ACCEPTANCE-REVIEW-PACKET.md)), this package folds proposed tasks P-06 (008-T19, remove `allow-same-origin` from the Browser preview iframe and add a guard test; audit finding U-02) and P-13 (008-T20, Git worktree view, VCS diff and user-invoked GitHub PR ops behind [Feature 006](../006-golden-connection-capability-write/SPEC.md)). Sources: [docs/UI-RENDERING-SETTINGS-ARCHITECTURE-AUDIT-2026-11.md](../../docs/UI-RENDERING-SETTINGS-ARCHITECTURE-AUDIT-2026-11.md) section 9 and [docs/REFERENCE-SYNTHESIS-AND-V1-FEATURE-ALIGNMENT-2026-11.md](../../docs/REFERENCE-SYNTHESIS-AND-V1-FEATURE-ALIGNMENT-2026-11.md) section 8.

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
