# 015 - Task Control and Heterogeneous Delegation

- **Status:** IN_REVIEW - authored complete for owner review; does not authorize implementation until the owner accepts the SPEC and blocker B-39.
- **Revision:** 0.1 | **Updated:** 2026-11-09 | **Blocker:** B-39 (first heterogeneous delegation pair and Environment sharing rule), PROPOSED, pending owner.
- **Owners:** Task Runtime, Delegation, Storage, Operator/API, Desktop UI (Work screen), Security/Trust and Product.
- **Release rows:** Linux x86_64 first; Windows x64 UNAVAILABLE until Feature 005 B-35 qualifies.

## User outcome

On the Work screen the owner can pause, resume, steer, cancel and hand off a running Task, see a truthful Activity timeline and Workers view, and let the lead agent delegate one eligible Step to a different native agent whose result an independent agent verifies. A late result from a replaced Attempt never changes the Task.

## Current truth

Child-admission decisions, lead-failover policy, lead-handoff packets and escalation policy exist as Rust domain code in `crates/domain-task` with no production caller. The only served pieces are the lead-binding change route (`POST /v1/tasks/{task_id}/lead-agent`, no new Attempt) and the read-only verification-runs list. Pause, resume, steer, Task-level cancel fan-out, worker dispatch, independent verification and the Work-screen timeline are absent. Every execution path depends on Feature 005, which qualifies on Linux first.

Under this package's IN_REVIEW status, proposed task P-14 is folded in as 015-T20: a Mission Control canvas (delegation tree, step status, live tool activity, approvals, evidence) bound to real runtime records after the first bounded workflow is proven, plus an explicitly-requested read-only Compare Agents fan-out. Source: [docs/REFERENCE-SYNTHESIS-AND-V1-FEATURE-ALIGNMENT-2026-11.md](../../docs/REFERENCE-SYNTHESIS-AND-V1-FEATURE-ALIGNMENT-2026-11.md) section 8.

## Package

- [SPEC.md](SPEC.md) - problem, decisions (B-39), HLD/LLD, security, EARS acceptance.
- [PLAN.md](PLAN.md) - phases, dependency graph, gates, risks.
- [TASKS.md](TASKS.md) - ordered tasks to VERIFIED for V1.
- [DATA-MODEL.md](DATA-MODEL.md) - records, lifecycles, fences, migration.
- [RESEARCH.md](RESEARCH.md) - pinned references (OpenHands) and decisions.
- [WIRING.yaml](WIRING.yaml) - components, edges, transactions, boundaries.
- [contracts/](contracts/README.md) - API/events, errors/recovery, UI behavior.

## Dependencies

Hard: Feature 005 (Attempt admission, Environment, cancel, recovery). Also Feature 002 and Feature 004 (Claude Code as second agent), Feature 009 (Coworker worker allowlist), Feature 011 (IPC), Feature 019 (release rows), Feature 020 (Needs You destination).

## Canonical owners

[Architecture](../../ARCHITECTURE.md), [V1 release](../../docs/V1-PRODUCT-RELEASE-CONTRACT.md), [Task Runtime](../../docs/TASK-RUNTIME.md), [Delegation](../../docs/DELEGATION.md), [State Machines](../../docs/STATE-MACHINES.md), [Environments](../../docs/ENVIRONMENTS.md), [Trust](../../docs/TRUST.md), [API](../../docs/API.md), [Events](../../docs/EVENTS.md), [Storage](../../docs/STORAGE.md), [Failure Recovery](../../docs/FAILURE-RECOVERY.md), [UI screen contract](../../docs/V1-UI-SCREEN-CONTRACT.md).
