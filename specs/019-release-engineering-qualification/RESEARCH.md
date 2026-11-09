# Research and Decision Log - Release Engineering and V1 Qualification

**Feature:** 019 | **Status:** IN_REVIEW - not owner-accepted; authorizes no implementation; not a claim of implementation or qualification.
**Checked:** 2026-11-09 - repository audit at HEAD 234c564 on `docs/audit-competitor-superset`; upstream Tauri documentation pages were located by web search on 2026-11-09 and are cited by URL, not pinned to a source SHA.

## 1. Repository sources inspected

| Source | Path | Finding | Limit |
|---|---|---|---|
| Tauri config | `apps/litecowork-ui/src-tauri/tauri.conf.json`, `tauri.bundle.conf.json`, `capabilities/default.json` | `targets: "all"`; CSP without `object-src`/`base-uri`/`frame-ancestors`; `externalBin` sidecar; `core:default` only | No signing, updater or installer options |
| Shell crate | `apps/litecowork-ui/src-tauri/Cargo.toml` | Separate `[workspace]`; deps `tauri 2`, `operator-ipc`, `rfd`, `windows-local-security` on Windows; no updater plugin | Root `cargo test --workspace` does not cover it |
| Build scripts | `apps/litecowork-ui/scripts/build-daemon-sidecar.mjs`, `run-tauri.mjs` | Validated sidecar build/copy; Linux defaults to deb and refuses AppImage | No signing hook |
| Gate script | `scripts/check.sh` | Pinned toolchains; Rust and Python stages with freshness checks; served-route audit | No feature validator, frontend, secret scan, packaging check |
| Release scripts | `scripts/v1_packaging_check.py`, `v1_platform_matrix.py`, `v1_resilience_matrix.py`, `v1_soak_harness.py`, `v1_secret_scan.py`, `v1_store_probe.py` | Truthful probe design (never invent PASSED) | Retired output paths; Unix-only pieces; Python store, not daemon |
| Packaging | `scripts/packaging/systemd/litecoworkd.service` | 0700 StateDirectoryMode, UMask 0077, hardening; never installed by repo | Linux only |
| IPC | `crates/operator-ipc/src/windows.rs`, `unix.rs`; `crates/windows-local-security/src/lib.rs` | Per-logon SID pipe DACL; Unix peer UID | Qualification owned by Feature 011 |
| Policy | `docs/ACCEPTANCE-AND-E2E-PLAN.md`, `docs/TESTING.md`, `docs/SECURITY.md`, `docs/NETWORK-SECURITY.md`, `docs/BENCHMARKS.md`, `docs/LITECOWORK-BUILD-TASKS.md` BT-00, BT-10.5, `docs/V1-PRODUCT-RELEASE-CONTRACT.md` sections 1 and 7 | Evidence ladder, bundle fields, stop/update gate, WSL2 rule, budgets | Targets only |
| Repository root | `ls`, `Cargo.toml`, `apps/litecowork-ui/package.json` | No `LICENSE`, no `license` fields, no `.github/` | - |

## 2. Pinned external references

| Ref | Source and revision | License | Use | Decision |
|---|---|---|---|---|
| REF-DEEPSEEK-HARNESS | [deepseek-ai/deepseek-harness](https://github.com/deepseek-ai/deepseek-harness/tree/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159) at `fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159` (desktop 0.2.2-rc.1), per [REFERENCE-IMPLEMENTATIONS.md](../../docs/REFERENCE-IMPLEMENTATIONS.md) | MIT | Admission lock, drain, recheck and positive graceful ack before quit/update; installer and update tests as test prompts | **Prototype** the update ordering against Feature 011's gate; do not copy Electron code |
| REF-ZCODE | [zai-org/ZCode](https://github.com/zai-org/ZCode/tree/a9fe401eede6e6e5e01be9048679c206fe6f02d9) at `a9fe401eede6e6e5e01be9048679c206fe6f02d9` (v3.17.2) | Apache-2.0 + NOTICE | Named bounded shutdown phases; desktop host supervision | **Prototype** cleanup-phase inventory only |
| REF-OPENLOOMI | [melandlabs/openloomi](https://github.com/melandlabs/openloomi/tree/2aca1016fa3065e2fee912afd0e94576d0483c66) at `2aca1016fa3065e2fee912afd0e94576d0483c66` | Apache-2.0 | Same-stack Tauri + Rust app: capability/permission manifests, lifecycle and close-behavior modules | **Study** capability manifest layout for T06; reject any consent-gated capture features |
| Tauri updater plugin | [v2 updater docs](https://v2.tauri.app/plugin/updater/) (located 2026-11-09) | Tauri MIT/Apache-2.0 | Signed static or dynamic manifest, embedded public key, `createUpdaterArtifacts` | **Adopt** the official plugin from Rust; pin the crate version in `src-tauri/Cargo.lock` at T11 |
| Tauri Windows signing | [Windows code signing](https://tauri.app/distribute/sign/windows/) (located 2026-11-09) | docs | `signCommand`, certificate thumbprint, timestamp URL; signing prevents SmartScreen "unknown publisher" blocks | **Adopt** `signCommand` with an HSM-backed service |
| Tauri Windows installer | [Windows installer](https://v2.tauri.app/distribute/windows-installer/) (located 2026-11-09) | docs | NSIS install modes (`currentUser`), WebView2 install modes, MSI via WiX on Windows hosts | **Adopt** NSIS per-user as the qualified V1 artifact |

The c7e9c05 archive matrix ([docs/archive/legacy-implementation-c7e9c05](../../docs/archive/legacy-implementation-c7e9c05/QUALIFICATION-MATRIX.md)) is history only; ACCEPTANCE section 7 requires re-running inventories from the live implementation.

## 3. Alternatives considered

| Topic | Options | Choice (PROPOSED) | Reason |
|---|---|---|---|
| Windows installer | NSIS per-user; MSI per-machine; MSIX | NSIS per-user qualified first, MSI built for managed deployment | No elevation; per-user state and pipe DACL; MSIX packaging changes file-system virtualization semantics the daemon has not been tested under |
| Update mechanism | Tauri updater; manual reinstall only; third-party updater service | Tauri updater on Windows, manual on Linux | Official, signature-verified, Rust-side; Linux in-app update adds package-manager conflicts |
| Signing key custody | Local PFX; CI secret PFX; HSM signing service | HSM signing service | A PFX in CI or on a laptop is a single-copy secret that can leak through logs or backups |
| Soak/resilience | Python `sqlite3` harness; Rust tests over `storage-sqlite` | Rust tests, Python as runner | Measures shipped code; runs on Windows |
| Dependency audit | cargo-deny; cargo-audit + license tool | cargo-deny (F-02) | One tool for advisories, licences, bans, sources |
| A11y audit | Extend `check-rendered-ui.mjs`; add axe-core | Decide at T16 (F-03) | Adding a dependency needs a documented reason (CONTRIBUTING) |

## 4. Unresolved questions

See SPEC section 17 (O-01 to O-06). Upstream claims in section 2 are documentation statements; the exact plugin crate version and behavior must be pinned and tested in T11 before relying on them.
