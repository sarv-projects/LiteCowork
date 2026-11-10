# 019 - Release Engineering and V1 Qualification

- **Status:** IN_REVIEW - authored complete for owner review on 2026-11-09; not owner-accepted; authorizes no implementation; not a claim of implementation or qualification.
- **Revision:** 0.1 | **Updated:** 2026-11-09 | **Blockers:** B-43 (signing identity and update channel), B-44 (licensing/packaging decision), both PROPOSED, pending owner.
- **Implementation state (2026-11-10):** CODE skeleton, nothing run on GitHub. DONE: T02, T14, T15. PARTIAL: T03 (CI workflow), T05 (secret scan), T06 (CSP audit), T08, T10 (Linux deb config and scripts, no deb built), T12 (updater state, not mounted), T16, T17, T18, T21, plus CODEOWNERS. BLOCKED/deferred as release dependent: T01, T09, T11, T13 (B-43, B-44). Plans no daemon migration.
- **Owners:** Release Engineering, Security, Desktop Shell, Runtime (Feature 011 interface), QA/Accessibility, Product.
- **Release matrix rows:** Windows 11/10 x64 + OpenCode v2.0.32 first; Linux x64 + OpenCode v2.0.32 second; WSL2 never counts.

## User outcome

The owner installs a signed LiteCowork build on Windows (then Linux), updates it only when the Runtime proves nothing is running, and reads a qualification matrix where every claimed cell links to CODE, SYSTEM and owner USER evidence for that exact row. V1 ships only after an explicit go/no-go that lists every package's VERIFIED evidence.

## Current truth

There is no CI (`.github/` is absent), no signing, no updater (`tauri-plugin-updater` is not a dependency), no LICENSE and no license fields. `tauri.conf.json` bundles `"targets": "all"`; `run-tauri.mjs` already limits Linux to deb and refuses AppImage. The release scripts in `scripts/` (`v1_packaging_check.py`, `v1_platform_matrix.py`, `v1_resilience_matrix.py`, `v1_soak_harness.py`, `v1_secret_scan.py`) exist but have no owner, are not gated, and several do not run or do not measure the shipped code on the Windows row. This package owns them from here.

## Package

- [SPEC.md](SPEC.md) - problem, goals, decisions B-43/B-44, HLD/LLD, security, EARS acceptance.
- [PLAN.md](PLAN.md) - phases, waves, dependency graph, rollout and gates.
- [TASKS.md](TASKS.md) - T01 to T23 including the V1 go/no-go.
- [DATA-MODEL.md](DATA-MODEL.md) - release rows, cells, bundles, update attempts and decision records.
- [RESEARCH.md](RESEARCH.md) - pinned sources, Tauri updater and signing references, alternatives.
- [WIRING.yaml](WIRING.yaml) - CI, release job, signing, updater, gate and matrix wiring.
- [contracts/](contracts/README.md) - API/command, errors/recovery and UI behavior.

## Canonical owners

[Architecture](../../ARCHITECTURE.md), [V1 release contract](../../docs/V1-PRODUCT-RELEASE-CONTRACT.md), [Acceptance and E2E plan](../../docs/ACCEPTANCE-AND-E2E-PLAN.md), [Testing](../../docs/TESTING.md), [Security](../../docs/SECURITY.md), [Network security](../../docs/NETWORK-SECURITY.md), [Benchmarks](../../docs/BENCHMARKS.md), [Build tasks](../../docs/LITECOWORK-BUILD-TASKS.md), [Local Operator IPC](../../docs/LOCAL-OPERATOR-IPC.md), [Runtime lifecycle](../../docs/RUNTIME-LIFECYCLE.md), Feature [011](../011-desktop-shell-ipc-settings-runtime/README.md).
