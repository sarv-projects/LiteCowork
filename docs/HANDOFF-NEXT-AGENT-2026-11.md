# Next-agent handoff, 2026-11-10

- **Repository:** LiteCowork (sarv-projects/litecowork)
- **Branch:** docs/audit-competitor-superset (remote branch tip e9abe46 before this handoff commit set)
- **Audience:** the next coding agent. Written by the agent that stopped work on 2026-11-10 at the owner's request.
- **Scope of this file:** the full operating picture: environment, every command and its baseline, git and PR state, the complete commit list since the audited base, per-package per-task status, the exact next tasks, pitfalls, open decisions, deferred work and every study document. Where a deeper source exists it is linked, and the per-task source of truth remains each `specs/NNN-*/TASKS.md`.
- **Evidence vocabulary (AGENTS.md section 5):** TARGET is normative design only; CODE is compiled or unit exercised at an identified revision; SERVED is an authenticated route plus the owning storage or service path mounted; INTEGRATED is a real native provider or harness working with a persisted product path; QUALIFIED is an exact OS, build, provider and auth route with CODE, SYSTEM and owner USER evidence including denial and recovery. A mock, schema, catalog entry, UI tile, probe, transport helper, unit test or mounted route alone is not product qualification. Nothing in this repository is INTEGRATED with a live agent run and nothing is QUALIFIED.
- **Hard rules:** `AGENTS.md` is read-only. Markdown, YAML and most source files are CRLF, keep them. No em dashes in new prose. Never stage `.agents/` or `repo_take/`. Stage by explicit file name, never `git add -A` or `git add .`. Do not push, reset, clean, rebase or drop stashes unless the owner asks. No fabricated success: if you did not run it, say so.

Contents: 1 Environment; 2 Commands and baselines; 3 Git, branch and PR state; 4 Commit list; 5 Per-package per-task status; 6 Next tasks in wave order; 7 Pitfalls; 8 Open owner decisions; 9 Deferred work and where to test it; 10 Study documents and the DeepSeek reuse map.

## 1. Environment

### 1.1 The file /projects/sandbox/.work/env.sh (full contents)

```sh
# LiteCowork sandbox build environment (source this before cargo/pnpm).
# Rustup dir and CARGO_HOME are read-only in this sandbox; the pinned 1.98.1
# toolchain is not installed, so we use the newest installed (1.92.0) via a
# writable CARGO_HOME seeded from the read-only cache, and a hand-built dbus
# pkg-config sysroot (keyring -> dbus-secret-service -> libdbus-sys needs it).
W=/projects/sandbox/.work
export CARGO_HOME=$W/cargo
export CARGO_TARGET_DIR=$W/target
export TMPDIR=$W/tmp
export RUSTUP_TOOLCHAIN=1.92.0
export PKG_CONFIG_PATH=$W/sysroot/usr/lib64/pkgconfig
export PKG_CONFIG_ALLOW_SYSTEM_LIBS=1
export PKG_CONFIG_ALLOW_SYSTEM_CFLAGS=1
export LIBRARY_PATH=/projects/sandbox/.work/libdir:/usr/lib64
export PYTHONPATH=$W/pylibs
export RUSTFLAGS="${RUSTFLAGS} -L /projects/sandbox/.work/libdir"
```

The file lives outside the repository (`/projects/sandbox/.work/`, a sibling of the checkout `/projects/sandbox/litecowork`). It is a sandbox artifact and is not committed. If the sandbox is rebuilt, recreate it from the block above. Note that `/tmp` in the sandbox is not persistent between tool calls; keep scratch files under `$W`.

### 1.2 Why each line exists

- **RUSTUP_TOOLCHAIN=1.92.0 versus the pinned 1.98.1.** `rust-toolchain.toml` pins 1.98.1. That toolchain is not installed in the sandbox and the rustup directory is read-only, so no download or install is possible. 1.92.0 is the newest installed toolchain. Consequences: three Python tests and `cargo clippy -D warnings` on the untouched `domain-effects` crate fail only because of the version difference; all code in this run compiles and tests on 1.92.0. Re-run clippy on 1.98.1 before any release claim.
- **CARGO_HOME=$W/cargo (writable, seeded from the read-only cache).** The default CARGO_HOME is read-only here, and cargo needs to write its lock files (`.package-cache`, `.package-cache-mutate`, `.global-cache`). `$W/cargo` was seeded from the read-only registry cache (`registry/cache`, `registry/index`, `registry/src`) so builds work offline.
- **CARGO_TARGET_DIR=$W/target.** Build output is kept outside the repository so it cannot be staged by accident.
- **TMPDIR=$W/tmp.** Several tests create temporary workspaces, SQLite files and sockets. They need a writable, roomy TMPDIR. Create it with `mkdir -p /projects/sandbox/.work/tmp` if missing.
- **PKG_CONFIG_PATH=$W/sysroot/usr/lib64/pkgconfig, PKG_CONFIG_ALLOW_SYSTEM_LIBS=1, PKG_CONFIG_ALLOW_SYSTEM_CFLAGS=1.** The OS credential-store dependency chain is `keyring` to `dbus-secret-service` to `libdbus-sys`. `libdbus-sys` runs pkg-config for `dbus-1`. The sandbox has no dbus development package, so a dbus pkg-config sysroot was hand-built under `$W/sysroot/usr/lib64/pkgconfig`. The two ALLOW variables stop pkg-config refusing system paths.
- **LIBRARY_PATH and RUSTFLAGS `-L`.** Linking needs `libdbus-1.so`. `$W/libdir/libdbus-1.so` is a hand-made link to the runtime library (the runtime package ships only the versioned `.so.3`). `LIBRARY_PATH` covers gcc link lookups and the RUSTFLAGS `-L` covers rustc link lookups, including build scripts and test binaries.
- **PYTHONPATH=$W/pylibs.** The documentation gate scripts need third-party Python packages (`jsonschema`, `attrs`, `yaml` and dependencies) that are not installed system-wide. They live in `$W/pylibs`. `scripts/validate_architecture.py` prints one warning that the optional `openapi-spec-validator` is not installed; that is expected.

### 1.3 Known environment limits

- `src-tauri` cannot be built here: the system lacks `gdk-3.0` development files. Every Tauri command added in this run is therefore uncompiled. Compile it on a machine with the Tauri Linux prerequisites or on the owner's Windows laptop.
- No Chrome or Chromium: no rendered UI run, no browser accessibility run, no Settings rendered-browser check.
- No GPG agent, no `dpkg-deb`, no `actionlint`: the signed checksum, deb build and workflow lint steps of Feature 019 are written but not exercised.
- The GitHub Actions workflow (Feature 019 T03) has never run on GitHub. A CI claim of any kind is unsupported until it does.
- No live OpenCode or Claude Code binary: no INTEGRATED claim exists, and 001-T09, 001-T12, 002-T12 and the O-05 input budget stay open.
- Network posture is repository access only for external hosts; `pnpm` uses the local store `/projects/sandbox/.pnpm-store`.

## 2. Commands and current baselines

Always run `source /projects/sandbox/.work/env.sh` in the same shell first. Run from the repository root `/projects/sandbox/litecowork` unless stated.

### 2.1 Rust

| Command | Baseline at tip e9abe46 | Notes |
|---|---|---|
| `cargo check --workspace` | 0 errors | `src-tauri` is not buildable here. |
| `cargo test -p litecowork-storage-sqlite --lib` | 624 passed, 0 failed (SCHEMA_VERSION 32) | Always use `--lib` (see pitfalls). Includes the v31 history and v32 co-edit storage tests. |
| `cargo test -p litecoworkd --bin litecoworkd` | 656 passed, 0 failed, 1 ignored | The one ignored test is the declared ignored daemon test. |
| `cargo fmt --all -- --check` | clean | Run before every commit. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 diagnostics on the pinned 1.98.1 toolchain | On 1.92.0 the untouched `domain-effects` crate fails only because of the version gap. |

Historical Windows full-workspace run (predates the Linux-side batches; superseded for counts): `cargo test --workspace --locked` exit 0 in 1,383.28 s with `CARGO_BUILD_JOBS=1`; litecoworkd 359 passed and 1 ignored, storage-sqlite 555, storage-core 110. Do not quote those counts as current.

### 2.2 UI (apps/litecowork-ui)

| Command (cwd apps/litecowork-ui) | Baseline | Notes |
|---|---|---|
| `pnpm test` | 5 pre-existing failures: four Needs You page tests and the esbuild lockfile test | Identical on the base commit. Not caused by this run. |
| `npx tsc --noEmit` | clean | |
| `pnpm run api:check` | clean | OpenAPI-derived client drift check. |

### 2.3 Documentation and contract gates (run from the repository root, in this order)

1. `python3 .specify/scripts/validate_feature_package.py` : 20 packages pass. Structural checks only, not owner approval or qualification.
2. `python3 scripts/audit_v1_design_depth.py` : 0 findings. Regenerates part of `docs/audit/`.
3. `python3 scripts/audit_v1_markdown.py` : 0 broken relative links. Regenerates `docs/audit/V1-MARKDOWN-AUDIT.md` and `docs/audit/V1-MARKDOWN-INVENTORY.csv`.
4. `python3 scripts/validate_design_owner_consistency.py` : cross-owner assertions, 0 issues.
5. `python3 scripts/validate_architecture.py` : passes with one warning (optional `openapi-spec-validator` not installed); JSON Schema, 149 typed events, errors, OpenAPI structure, SQLite, Gateway naming, canonical concepts and active Markdown checks pass.
6. `python3 scripts/audit_v1_contracts.py` : 0 errors. Regenerates `docs/audit/v1-cross-contract-guard.json`.
7. `python3 scripts/build_active_corpus_inventory.py` : regenerates `docs/COMPLETE-ACTIVE-CORPUS-INVENTORY.md` (LF endings as before).

Also available: `python3 scripts/audit_served_routes.py` regenerates and checks `docs/schemas/served-route-inventory.json`. Baseline route inventory: **178 OpenAPI served / 1 authenticated local-only / 53 target-only** (231 OpenAPI operations). Also `scripts/check.sh --preflight` (the full local gate wrapper) and `scripts/v1_secret_scan.py`.

Gate output churn: gates 2, 3 and 6 rewrite `docs/audit/*` with LF endings while the committed copies are CRLF. After the gates, record the counts in CURRENT-RUN.md, docs/IMPLEMENTATION-STATUS.md and docs/DOCS-REAUDIT-2026.md, then run `git checkout -- docs/audit` last. Keep the regenerated `docs/COMPLETE-ACTIVE-CORPUS-INVENTORY.md`. Validator quirk: `validate_feature_package.py` fails if any full `NNN-TNN` task ID appears twice in one package TASKS.md; prose notes therefore use the short form "task T17 of package 011".

### 2.4 Gate counts at this handoff

The authoritative numbers are in the gate output and in the "Current validation" lines of CURRENT-RUN.md and docs/IMPLEMENTATION-STATUS.md (reconciled by the final step of this handoff commit set): 20 feature packages validate, 0 broken links, route inventory 178/1/53, 149 typed events.

## 3. Git, branch and PR state

- Working branch: `docs/audit-competitor-superset`. Remote branch `origin/docs/audit-competitor-superset` is at e9abe46 (pushed before this handoff set).
- **PR #1** ([sarv-projects/litecowork#1](https://github.com/sarv-projects/litecowork/pull/1), "docs: assumption-free corpus re-audit + fresh competitor deep-dive + superset gap analysis"): state **closed and merged** into `main` on 2026-11-09 (merge commit ced0e6f, PR head 06620c7, base 2e85f03, 15 commits, 281 changed files).
- **PR #2** ("Docs/audit competitor superset", head 23e9edb): also **closed and merged**.
- Everything after 23e9edb on this branch (the V1 implementation commits and this documentation sync) is **not in any open PR**. Because both PRs are merged, open a **new PR** (new branch name or this branch against `main`, as the owner prefers). Create PRs with `gh api repos/sarv-projects/litecowork/pulls ...`; the `gh pr` subcommands do not work in this sandbox. Check `gh api "repos/sarv-projects/litecowork/pulls?state=all&per_page=10"` first.
- The local `origin/main` ref may be stale (showed 2e85f03); `git ls-remote origin` showed the real `main` (5aa0e09 at inspection). `git fetch` before comparing.
- Stash: `stash@{0}` "wip: v28 delegation-snapshot fixture fixes (lead_failover, environment) - apply with v28 resolution" touches `crates/storage-sqlite/src/environment_tests.rs` and `crates/storage-sqlite/src/lead_failover_choice_request.rs`. It is **probably obsolete**: v28 is wired with its writer and every fixture now supplies a snapshot. Do not drop it until `git stash show -p stash@{0}` has been compared with those two files and the same fix is confirmed present. Do not `git stash pop` blindly.
- CI has never run on GitHub (see 1.3).
- The owner is `sarv-projects`. The owner's native Windows x64 laptop is the first qualification machine for deferred rows (section 9).

## 4. Commit list since the audited base (2e85f03..HEAD)

Each entry is the commit subject (what it did) with its change statistics. For file lists run `git show <sha> --stat`.

Count: 78 commits from 2e85f03 (exclusive) to e9abe46 (inclusive), oldest first. Statistics are files changed, insertions, deletions from `git show --shortstat`.

- `33553d9` (2026-11-08) chore(docs-audit): establish audit baseline and gate snapshot. [7 files changed, 289 insertions(+)]
- `a13b936` (2026-11-08) docs(audit): phase 1 assumption-free corpus re-audit, register A-51, drift reconciliation. [24 files changed, 1272 insertions(+), 1104 deletions(-)]
- `31ec8d4` (2026-11-08) docs(competitors): phase 2 fresh rival re-verification, SHAs/runtimes/borrowable URLs, RV-13..RV-14. [13 files changed, 560 insertions(+), 103 deletions(-)]
- `0025a9f` (2026-11-08) docs(gap): phase 3 superset matrix + three-way gap analysis. [11 files changed, 251 insertions(+), 43 deletions(-)]
- `948dd4f` (2026-11-08) docs(audit): address review v1 - reconcile corpus counts, OpenBot pin, UIV1 provenance, register ordering. [14 files changed, 61 insertions(+), 53 deletions(-)]
- `ada9481` (2026-11-08) docs(owners): phase A - resolve open findings A-18/A-24/A-26/A-34/A-20/A-35/A-37 with PROPOSED owner resolutions. [9 files changed, 97 insertions(+), 16 deletions(-)]
- `4fa41ef` (2026-11-08) docs(specs): deepen 002-006 to 001 bar, PROPOSED resolve B-11..B-20, remove template artifact, fold research. [19 files changed, 771 insertions(+), 156 deletions(-)]
- `d117ac1` (2026-11-08) docs(specs): deepen 007-011 to 001 bar, PROPOSED resolve B-21..B-33, remove template artifact, fold research. [19 files changed, 784 insertions(+), 122 deletions(-)]
- `b5fdbe7` (2026-11-08) feat(specs): add package 012 conversation co-edit/take-control (RV-14), register + depth-gate count. [18 files changed, 670 insertions(+), 8 deletions(-)]
- `46415c0` (2026-11-08) chore(docs): regenerate inventories and reconcile corpus counts after spec upgrade; all gates green. [6 files changed, 812 insertions(+), 802 deletions(-)]
- `2e49615` (2026-11-08) docs(specs): address v1 review - remove em dashes in specs/README, add missing EARS categories to 007/009/011. [11 files changed, 69 insertions(+), 37 deletions(-)]
- `8129420` (2026-11-09) fix(ui): resolve A-24 Send qualification gate and A-26 empty-state branding. [21 files changed, 810 insertions(+), 668 deletions(-)]
- `234c564` (2026-11-09) docs: add owner acceptance review packet for all PROPOSED resolutions. [9 files changed, 120 insertions(+), 31 deletions(-)]
- `e678a91` (2026-11-09) docs(specs): owner acceptance of 001-012, V1 release task plans, new packages 013-020. [238 files changed, 17586 insertions(+), 2041 deletions(-)]
- `06620c7` (2026-11-09) docs(audit): agent-output rendering, Intelligent UI, UI, Settings and architecture audit. [11 files changed, 242 insertions(+), 40 deletions(-)]
- `16352fd` (2026-11-09) docs(research): ACP transport direction, LiteSPM-as-service and reference synthesis. [12 files changed, 189 insertions(+), 43 deletions(-)]
- `dcc5142` (2026-11-09) docs: fold owner-approved tasks, product-first README, feature-coverage answer, LiteSPM integration study. [53 files changed, 977 insertions(+), 341 deletions(-)]
- `44c691a` (2026-11-09) docs: propagate resolved LiteSPM pin (84d5fab) to owner docs and Feature 013. [14 files changed, 61 insertions(+), 59 deletions(-)]
- `23e9edb` (2026-11-09) docs(013): correct stale 'LiteSPM not reachable' claim; repo was cloned and pinned at 84d5fab. [5 files changed, 9 insertions(+), 9 deletions(-)]
- `3bbaf6d` (2026-11-09) docs(research): deep-dive of 8 rival plugin/capability architectures. [11 files changed, 153 insertions(+), 40 deletions(-)]
- `8e4cf80` (2026-11-09) docs(research): line-level deep-dive of DeepSeek Harness plugin architecture. [10 files changed, 124 insertions(+), 36 deletions(-)]
- `01f525a` (2026-11-09) docs(research): line-level deep-dives of open-design, ZCode and paradev. [12 files changed, 168 insertions(+), 39 deletions(-)]
- `e10d496` (2026-11-09) feat(001/002): ACP native-agent adapter, event normalizer and turn fixture. [22 files changed, 16419 insertions(+), 14189 deletions(-)]
- `78fba68` (2026-11-09) feat(001): wire v29 conversation dispatch migration, OpenAPI commit-identity, ACP RESEARCH pins. [5 files changed, 176 insertions(+), 10 deletions(-)]
- `ef2bed7` (2026-11-09) fix(001): defer migration wiring; keep ACP code, restore green baseline. [16 files changed, 183 insertions(+), 697 deletions(-)]
- `a261958` (2026-11-09) docs: update CURRENT-RUN with V1 implementation progress; fix stale package counts. [9 files changed, 57 insertions(+), 43 deletions(-)]
- `08ae5bd` (2026-11-09) feat(011): IPC admission bound tests (T04). [3 files changed, 198 insertions(+), 4 deletions(-)]
- `bc6e8cb` (2026-11-09) feat(011): truthful runtime health rows in Settings (T05). [5 files changed, 126 insertions(+), 7 deletions(-)]
- `ff412a6` (2026-11-09) feat(011): promote stop preview, rename, keep-awake contracts (T01). [20 files changed, 258 insertions(+), 21 deletions(-)]
- `9a40170` (2026-11-09) feat(011): applied and persisted local display preferences (T07). [9 files changed, 253 insertions(+), 47 deletions(-)]
- `8b9caa4` (2026-11-09) feat(011): Settings control map and inert-control audit (T06 partial). [6 files changed, 124 insertions(+), 24 deletions(-)]
- `32e38f8` (2026-11-09) feat(011): Workspace rename over Operator, Tauri and Settings (T08). [27 files changed, 864 insertions(+), 39 deletions(-)]
- `2611c4f` (2026-11-09) feat(011): Notifications link and probed-only Agents heading (T10, T09 partial). [8 files changed, 108 insertions(+), 8 deletions(-)]
- `dccd35d` (2026-11-09) feat(011): shortcut registry and shell reachability guard (T11, T19 partial). [6 files changed, 138 insertions(+), 27 deletions(-)]
- `263191b` (2026-11-09) feat(019): CI skeleton, LF pins, secret scan masking and exceptions (T03, T05 partial). [16 files changed, 892 insertions(+), 513 deletions(-)]
- `2f0c73e` (2026-11-09) docs(011): record deferrals and Settings status (T18 partial). [2 files changed, 5 insertions(+), 5 deletions(-)]
- `d6dda97` (2026-11-09) feat(019): CSP hardening and audit script (T06 partial). [5 files changed, 164 insertions(+), 2 deletions(-)]
- `79f72fc` (2026-11-09) feat(019): release evidence and row schemas, release docs, ADR skeleton (T02). [11 files changed, 329 insertions(+), 5 deletions(-)]
- `d3dbad1` (2026-11-09) feat(019): Linux deb config, signed checksum and package qualification scripts (T10 partial). [5 files changed, 220 insertions(+), 2 deletions(-)]
- `84a58c6` (2026-11-09) feat(019): update panel state, controller and component (T12 partial); CRLF for new files. [11 files changed, 978 insertions(+), 364 deletions(-)]
- `67ded7c` (2026-11-09) feat(019): row-aware release qualification matrix (T14). [7 files changed, 888 insertions(+), 19 deletions(-)]
- `58a7185` (2026-11-09) feat(019): evidence bundle tool, redacted index and USER sign-off template (T15). [5 files changed, 451 insertions(+), 5 deletions(-)]
- `19e9a87` (2026-11-09) feat(019): native storage resilience tests and portable runner (T18 partial). [4 files changed, 227 insertions(+), 5 deletions(-)]
- `974e962` (2026-11-09) feat(011): serve read-only stop preview with honest UNKNOWN verdict (T13 prerequisite). [22 files changed, 649 insertions(+), 47 deletions(-)]
- `a2ca4ad` (2026-11-09) feat(011): stop preview panel and update quiescence gate (T14 partial). [10 files changed, 607 insertions(+), 6 deletions(-)]
- `981e82e` (2026-11-09) feat(011): map every AgentCatalogSettings control and pin action states (T09). [5 files changed, 184 insertions(+), 34 deletions(-)]
- `be75fa5` (2026-11-09) feat(011): skip link, universal focus-visible, registry shortcut list (T11 partial). [6 files changed, 64 insertions(+), 3 deletions(-)]
- `3539ec9` (2026-11-09) feat(011): keep-awake power assertion state machine and Linux backend (T12 partial); fmt. [4 files changed, 402 insertions(+), 6 deletions(-)]
- `fb3caef` (2026-11-09) feat(011): merge SettingsPage into SettingsView, render More nav group, fix false reachability test (T19, T21). [7 files changed, 187 insertions(+), 256 deletions(-)]
- `ce2c8a7` (2026-11-09) refactor(011): shared Operator-unavailable panel and LibraryPage module (T20 partial). [6 files changed, 874 insertions(+), 828 deletions(-)]
- `431c397` (2026-11-09) feat(011): Workspace Dashboard with pinnable widgets over served projections (T22 partial). [6 files changed, 335 insertions(+), 5 deletions(-)]
- `2c2cab0` (2026-11-09) docs(011): cross-layer trace table with verified symbols and gaps (T16 partial); fix duplicate task id refs. [2 files changed, 26 insertions(+), 2 deletions(-)]
- `5f1c492` (2026-11-09) test: update route-count expectations after serving stop preview (175/1/55). [1 file changed, 2 insertions(+), 2 deletions(-)]
- `39a9e28` (2026-11-09) docs: record Wave 0 release machinery and shell progress honestly. [5 files changed, 120 insertions(+), 77 deletions(-)]
- `202a8f6` (2026-11-09) feat(001): wire v28 and v29 migrations; durable dispatch outbox, single-flight admission, response commit identity (T01,T02,T10,T11). [40 files changed, 1396 insertions(+), 142 deletions(-)]
- `00c4542` (2026-11-10) feat(001): effective Stop and Retry lifecycle, durable turn settlement, ACP adapter work; mark Runtime DEGRADED on Operator endpoint failure and fix stale runtime process test expectations. [17 files changed, 1544 insertions(+), 304 deletions(-)]
- `9c734b5` (2026-11-10) docs: record owner decisions on packet Part 4 (A-62); defer LiteSPM, Windows and release-dependent rows. [25 files changed, 86 insertions(+), 84 deletions(-)]
- `6ff93dc` (2026-11-10) docs: regenerate active corpus inventory after owner decision record. [1 file changed, 36 insertions(+), 36 deletions(-)]
- `b5a3e69` (2026-11-10) feat(001): restart and orphan recovery sweep for Conversation turns (T08). [6 files changed, 220 insertions(+), 1 deletion(-)]
- `6c2af2c` (2026-11-10) feat(001): bounded transient turn.delta and turn.settled frames on the Conversation stream (T13). [8 files changed, 620 insertions(+), 10 deletions(-)]
- `f998f61` (2026-11-10) feat(001): B-07 sliding context window with visible omission notice; real adapter budget stays unconfirmed (T05). [7 files changed, 304 insertions(+), 33 deletions(-)]
- `6a312b0` (2026-11-10) feat(001): usage records at every turn settlement with reported, estimated and unknown attribution (T15). [8 files changed, 253 insertions(+), 16 deletions(-)]
- `5f83d92` (2026-11-10) refactor(001): AgentTurnCoordinator as the single dispatch entry and redacted dispatch diagnostics (T20). [4 files changed, 106 insertions(+), 33 deletions(-)]
- `ec34a98` (2026-11-10) feat(001): Stop and Retry bound to projected turn versions, composer fence, honest controls and stick-to-bottom autoscroll (T16, T17, T18, T26). [15 files changed, 599 insertions(+), 49 deletions(-)]
- `f2090ee` (2026-11-10) feat(001): migration v30 REPORTED turn activity records, storage layer of T25; record LiteSPM and Windows deferral. [11 files changed, 293 insertions(+), 24 deletions(-)]
- `0edc802` (2026-11-10) feat(001): REPORTED turn activity emission, served read route and UI activity cards (T25). [30 files changed, 829 insertions(+), 45 deletions(-)]
- `18c7477` (2026-11-10) feat(001): text ResourceRef materialization at submit and dispatch, composer attachment picker (T14 partial). [16 files changed, 672 insertions(+), 21 deletions(-)]
- `2e36670` (2026-11-10) docs(001): record T12 blocked finding, T27 display-only permission card, T21 cross-layer trace, T22 status sync. [11 files changed, 59 insertions(+), 30 deletions(-)]
- `af3b641` (2026-11-10) feat(002): OpenCode AgentModule manifest, descriptor store and served descriptor routes, fresh-descriptor enable gate (T01 sync, T03, T04, T05 partial, T06 partial). [19 files changed, 1797 insertions(+), 79 deletions(-)]
- `bf9b8e0` (2026-11-10) feat(002): ACP client adapter with bounded framing, cancel and fail-closed faults (T17). [2 files changed, 705 insertions(+)]
- `ee26522` (2026-11-10) feat(002): served beginAgentAuthentication and openAgentNativeConfiguration, auth observation service, composer model picker view-model (T07, T09, T10 partial). [9 files changed, 1107 insertions(+), 6 deletions(-)]
- `44b6298` (2026-11-10) docs: DeepSeek Harness desktop shell deep-dive and register A-63. [2 files changed, 385 insertions(+)]
- `de165e1` (2026-11-10) feat(002): Matchmaker ranking engine, descriptor panel axes and declared sign-in/native-config, install-unavailable notice, restart recovery tests (T10, T11, T13, T18 partial). [7 files changed, 417 insertions(+), 4 deletions(-)]
- `b28174c` (2026-11-10) docs(002): sync task statuses, route inventory 178/1/53, cross-layer trace, DEFERRED and BLOCKED notes (T08, T12, T14, T15, T16). [15 files changed, 100 insertions(+), 67 deletions(-)]
- `fb99570` (2026-11-10) feat(003): workspace root lifecycle storage tests (T02 partial). [4 files changed, 187 insertions(+), 12 deletions(-)]
- `340912b` (2026-11-10) docs: second pass on DeepSeek desktop deep-dive with line-level scripts, src and installer findings. [1 file changed, 253 insertions(+), 34 deletions(-)]
- `9bdde84` (2026-11-10) docs: update register A-63 read depth and proposed tasks after second pass. [1 file changed, 1 insertion(+), 1 deletion(-)]
- `e9abe46` (2026-11-10) feat(014,012): conversation history lifecycle and co-edit control storage (migrations v31, v32), storage layer only; renumber later migrations. [21 files changed, 1584 insertions(+), 78 deletions(-)]

Commits after e9abe46 are the 2026-11-10 documentation sync set (documentation only): package status and migration renumber sweep with DEFERRED/BLOCKED notes; the DeepSeek Harness reuse map with permalinks and PROPOSED rows; this handoff. List them with `git log --oneline e9abe46..HEAD`.

Groupings for orientation: e10d496, ef2bed7 ACP native adapter and green-baseline restore; ff412a6 to 3539ec9 and 2c2cab0 Feature 011 shell tasks; 263191b to 19e9a87 Feature 019 CI and release machinery; 202a8f6 to 18c7477 Feature 001 durable turn (migrations v28, v29, v30); af3b641 to b28174c Feature 002; fb99570 Feature 003 T02; e9abe46 Features 014 and 012 storage layers (v31, v32); the remaining docs commits are study, audit, owner acceptance and register updates.

## 5. Per-package per-task status

Generated from the `**Status:**` line of every task in `specs/NNN-*/TASKS.md` at tip e9abe46 plus the sync. The evidence label is derived from the status line using AGENTS.md section 5: DONE, PARTIAL or CODE-COMPLETE means CODE (and SERVED where the status line says the route is served); NOT STARTED, BLOCKED and DEFERRED mean TARGET only. No task is INTEGRATED or QUALIFIED. The detail text is the first 330 characters of the status line; the complete text, the cross-layer trace and the DEFERRED/BLOCKED note (why, what unblocks it, where to test later) are in the TASKS.md task. Rows named PROP-Pnn in TASKS.md are PROPOSED DeepSeek-derived items, not tasks (section 10).

### 001-conversation-session-supervisor

Package status: ACCEPTED/LOCKED. Source: `specs/001-conversation-session-supervisor/TASKS.md`.

- **001-T01** Record owner acceptance and promote accepted contracts into owner docs and schemas. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - owner acceptance of B-01 to B-10 recorded 2026-11-09. Promoted and implemented in this slice: `CONVERSATION_TURN_IN_PROGRESS` (error-codes and domain-event schemas, SCHEMAS.md, API.md), migration `docs/schemas/sqlite-v29.sql` (dispatch intents, response commit identity, usage records, single-flight triggers; ... (rest in TASKS.md)
- **001-T02** Harden the atomic final Agent response commit (B-04 caveat). Status: DONE. Evidence: CODE. Detail: DONE (storage) - `CommitConversationAgentResponse` now carries `retry_ordinal`, `provider_response_id` and optional usage; the IMMEDIATE transaction verifies the ordinal, the AGENT author pinned to the session binding, derives the receipt identity from `(turn_id, retry_ordinal)` with a content digest (exact repeat replays the ... (rest in TASKS.md)
- **001-T03** Complete bounded AgentSessionSupervisor and AgentHostSupervisor lifecycle. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - per-turn `AgentSessionSupervisor` in `apps/litecoworkd/src/agents/supervisor.rs` reserves STARTING and activates; it also creates the host record and starts/stops `OpenCodeServer` itself, so host and session ownership are not separate, there is no idle TTL, and the Job Object/cgroup lifetime control from B-09 is ... (rest in TASKS.md)
- **001-T04** Qualify the OpenCode v2.0.32 AgentAdapter behind the Adapter 1.0 contract. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - the provider-neutral `AgentAdapter` trait (probe_version, create_session, send_context, drive_turn, interrupt, shutdown), the OpenCode v2.0.32 `OpenCodeAdapter` (wraps `OpenCodeServer`, `interrupt` calls `abort_session`), the ACP client spine and per-provider launch catalog exist in ... (rest in TASKS.md)
- **001-T05** Complete bounded context dispatch and confirm the first adapter budget (B-07 caveat). Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (window implemented; real OpenCode budget NOT confirmed) - the B-07 window now exists in two layers. `build_conversation_context_package` no longer refuses a long Conversation: it keeps the newest contiguous messages ending at the boundary within 1,024 messages and 4 MiB and records the older count in ... (rest in TASKS.md)
- **001-T06** Wire final response settlement with stable identity in the supervisor. Status: DONE. Evidence: CODE. Detail: DONE (without a live native adapter) - the supervisor derives the commit request id, message id and `provider_response_id` deterministically from `(turn_id, retry_ordinal, native session/message)`, takes expected versions from the activation result instead of hard-coded 2/3, and retries only transient store failures through ... (rest in TASKS.md)
- **001-T07** Make Stop effective with observed quiescence and fix the Retry lifecycle (daemon). Status: DONE. Evidence: CODE + SERVED. Detail: DONE (daemon + storage; native interrupt covered by adapter fixtures, not a live agent) - Cancel on RUNNING commits only CANCEL_REQUESTED and signals the attempt through `AgentSessionSupervisor::request_cancel` (per `(turn_id, retry_ordinal)` `CancelSignal`); the OpenCode drive loop sends one `abort_session` interrupt and ... (rest in TASKS.md)
- **001-T08** Implement daemon restart and orphan recovery sweep. Status: DONE. Evidence: CODE. Detail: DONE (restart sweep; crash windows covered with seeded rows, no live agent kill) - `agents/conversation_recovery.rs` runs from `recover_and_persist_startup_blockers` before the Operator accepts commands. It settles every UNCLAIMED or CLAIMED dispatch intent's turn FAILED with reason `RUNTIME_RESTARTED` (RUNNING, ... (rest in TASKS.md)
- **001-T09** Qualify the end-to-end native Conversation route on the release matrix and gate Send. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - no `apps/litecoworkd/tests/conversation_session_supervisor.rs`; no Windows or Linux SYSTEM/USER evidence.
- **001-T10** Add a durable dispatch outbox for admitted turns. Status: DONE. Evidence: CODE. Detail: DONE - `conversation_dispatch_intents` (v29) is written in the same transaction as the turn by `create_conversation_turn`, claimed by compare-and-swap (`claim_conversation_dispatch_intent`) before the STARTING reservation in `AgentSessionSupervisor::dispatch_turn`, settled by the final-response commit and by any terminal turn ... (rest in TASKS.md)
- **001-T11** Enforce single-flight admission in storage (B-01). Status: DONE. Evidence: CODE. Detail: DONE - `create_conversation_turn` rejects a second turn while another is OPEN, RUNNING, WAITING_USER, WAITING_DEPENDENCY or CANCEL_REQUESTED with `CONVERSATION_TURN_IN_PROGRESS` after the receipt replay check; v29 triggers back it at the database level (triggers rather than a partial unique index so legacy duplicate rows still ... (rest in TASKS.md)
- **001-T12** Wire Conversation-scoped UserRequest continuation (B-06). Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (provider capability not evidenced; path recorded unavailable per the Stop rule) - the domain WAITING_USER/WAITING_DEPENDENCY transitions and `user_request_operator.rs` exist, but the OpenCode v2.0.32 normalizer maps no input-request event and neither the captured fixture nor any repository source shows the pinned ... (rest in TASKS.md)
- **001-T13** Emit bounded transient TURN_DELTA frames with resync (B-05). Status: DONE. Evidence: CODE. Detail: DONE (daemon; UI reducer already handled the frames) - `apps/litecoworkd/src/transient_frames.rs` holds a fixed 256-frame broadcast ring (`transient_bus()`); `TurnFramePublisher` is attached to the OpenCode adapter as a `TurnObserver` and coalesces answer text into `turn.delta` frames (flush at 1 KiB or 100 ms, at most 8 KiB ... (rest in TASKS.md)
- **001-T14** Composer attachments and B-08 resource materialization. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (text attachments CODE-level; binary UNAVAILABLE; no Windows row; no live run) - `agents/resource_materializer.rs` reauthorizes every `resource_refs` entry and `RESOURCE` content block at dispatch (same-Workspace identity, Resource exists, pinned revision still the head, not RESTRICTED, allowlisted text, at most 128 KiB ... (rest in TASKS.md)
- **001-T15** Write usage records at turn settlement (B-10). Status: DONE. Evidence: CODE. Detail: DONE (OpenCode path; table was v29) - every attempt settlement writes exactly one `conversation_turn_usage_records` row inside its own transaction: the final commit (T02) and now `settle_conversation_turn` (FAILED and CANCELLED, `INSERT OR IGNORE`, `SettleConversationTurn.usage`). The supervisor attributes PROVIDER_REPORTED ... (rest in TASKS.md)
- **001-T16** Bind Stop to the projected active turn and real aggregate version. Status: DONE. Evidence: CODE. Detail: DONE (prerequisite of T17/T18, done with them; rendered DOM not exercised) - the projected `active_turn` now carries `version` and the snapshot gains `retryable_turn` (OpenAPI `ConversationTurnPresentation` and `ConversationRetryableTurn`, stream schema, generated `operator-api.ts`). `conversation-turn-state.ts` `stopTarget` ... (rest in TASKS.md)
- **001-T17** Add the failed-turn Retry UI. Status: DONE. Evidence: CODE. Detail: DONE (logic and source-level guards; no rendered test) - the latest FAILED turn renders an inline card with a fixed safe reason (`retryReasonLabel`; unknown codes never leak raw text), says the message was kept, and a Retry button whose accessible name includes the reason. It invokes `retry_conversation_turn` with the projected ... (rest in TASKS.md)
- **001-T18** Fence the composer for every nonterminal state and remove inert or fabricated composer controls. Status: DONE. Evidence: CODE. Detail: DONE (logic and source-level guards; no rendered test) - `isSendEligible` and `isTurnOccupyingComposer` in `conversation-turn-state.ts` fence OPEN, RUNNING, WAITING_USER, WAITING_DEPENDENCY and CANCEL_REQUESTED plus a visible streamed turn and the in-flight submit (so the gap after the 202 is covered). Attachments, Voice input ... (rest in TASKS.md)
- **001-T19** Keep the Send disabled-until-qualified UI gate and truthful empty state. Status: DONE. Evidence: CODE. Detail: DONE (UI gate only) - commit 8129420: `sendEligible` / `bindingEligibleForSend` in `apps/litecowork-ui/src/conversations/ConversationsPage.tsx` gate the `sendTurn` guard and `disabled={!sendEligible}`; the empty state is LiteCowork-branded with no unsupported claims; guarded by ... (rest in TASKS.md)
- **001-T20** Fix the stale Operator header and extract the shared dispatch admission path. Status: DONE. Evidence: CODE. Detail: DONE - the module header now describes the OpenCode-only dispatch path; `agents/turn_coordinator.rs` adds `AgentTurnCoordinator::dispatch_admitted_turn`, the single entry both submit and retry use (no handler calls `dispatch_turn` or `tokio::spawn` directly). The coordinator drives the supervisor, which claims the durable ... (rest in TASKS.md)
- **001-T21** Record the cross-layer trace for each user-visible operation. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL (trace recorded for Send, Stop, Retry, Reload, Activity and Attach; Answer UserRequest and native permission rows are recorded as gaps) - `contracts/ui-behavior.md` section 9 holds the rows with the exact control, Tauri command, route, fence, recovery and evidence test. Per the Stop rule the missing hops are open tasks ... (rest in TASKS.md)
- **001-T22** Synchronize status documents and inventories. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL (rows synced to current CODE evidence; closure waits on T09) - the served-route inventory was regenerated (176/1/55 at that time; current 178/1/53) and `audit_served_routes.py` matches it; `docs/IMPLEMENTATION-STATUS.md` has a 2026-11-10 Conversation status paragraph, register A-25 and A-26 carry dated updates but stay ... (rest in TASKS.md)
- **001-T23** Capture a redacted OpenCode v2.0.32 turn fixture (P-00). Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - `apps/litecoworkd/src/agents/fixtures/opencode_v2_0_32_turn.jsonl` replays deterministically and `opencode_turn_fixture_*` tests assert no handle, credential or prompt body survives. R-01 is not yet confirmed or refuted in RESEARCH.md and no OpenCode binary exists in this environment to re-capture a live v2.0.32 turn, ... (rest in TASKS.md)
- **001-T24** OpenCodeEventNormalizer with correlated internal events (P-01). Status: DONE. Evidence: CODE. Detail: DONE (fixture-driven) - `apps/litecoworkd/src/agents/normalizer.rs` (`OpenCodeEventNormalizer`) maps events to correlated internal events, counts and drops unknown, foreign-session and uncorrelated frames, never stores reasoning in the answer, and the supervisor consumes it through `drive_turn` with an idle/heartbeat budget. ... (rest in TASKS.md)
- **001-T25** Transient turn.activity frames and REPORTED activity records with UI cards (P-02). Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL (CODE + SERVED; no live OpenCode run, no rendered UI test) - migration v30 stores REPORTED records (`append_conversation_turn_activity`, `list_conversation_turn_activity`). The supervisor now attaches a `TurnActivityRecorder` observer next to the delta publisher and usage collector: for tool start and settle, permission ... (rest in TASKS.md)
- **001-T26** Stick-to-bottom autoscroll and new-messages indicator (P-04). Status: DONE. Evidence: CODE. Detail: DONE (logic and source wiring; no rendered scroll test) - `isAtBottom` (48 px threshold), `scrollFollowDecision` and `contentSignature` are pure and tested. The page follows new messages and streamed text only while the reader is at the bottom and not selecting text; otherwise it keeps the scroll position and shows a `New ... (rest in TASKS.md)
- **001-T27** Surface native permission prompts inline and in Needs You (P-09). Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (display only; answer channel BLOCKED) - `PermissionAsked` is persisted and framed as a REPORTED PERMISSION activity (T25) and the activity card says the agent asked for permission, that LiteCowork is not asking and cannot relay an answer yet, with no answer control. Not done: answering through the harness (the ACP ... (rest in TASKS.md)
- **001-T28** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.

### 002-native-agent-install-auth-binding

Package status: ACCEPTED/LOCKED. Source: `specs/002-native-agent-install-auth-binding/TASKS.md`.

- **002-T01** OpenCode v2.0.32 profile produces a bindable, dispatch-consistent offer. Status: CODE-COMPLETE. Evidence: CODE. Detail: CODE-COMPLETE 2026-11-10 (offer admission computed in `agents/native_provider.rs`, `NativeProviderKey`, shared executable resolver, OpenAPI `profile_admission_supported` boolean and generated types synced; probe, native_provider_key and opencode_profile tests pass). Open: Windows x64 and Linux x64 probe-to-binding transcripts ... (rest in TASKS.md)
- **002-T02** Promote the accepted 002 interfaces into owner docs and machine schemas. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (no migration was needed: the Runtime-local descriptor table already exists since migration 16, so no `sqlite-vNN.sql` was added; `EnableBindingWithDescriptor` is promoted as the optional `X-Agent-Descriptor-Revision` header on `enableAgentBinding` in the OpenAPI with `RESOURCE_STALE` for DescriptorStale; probe budget ... (rest in TASKS.md)
- **002-T03** Wire AgentModuleRegistry with the OpenCode v2.0.32 manifest and conformance fixture. Status: CODE-COMPLETE. Evidence: CODE. Detail: CODE-COMPLETE 2026-11-10 (OpenCode 2.0.32 manifest in `agents/modules/opencode.rs` with TTL 900 s and probe budget 3 per minute fields validated by the registry, registered through `default_module_registry()`, pinned fixture `apps/litecoworkd/tests/fixtures/agent-modules/opencode-2.0.32.json`; the registry file keeps a reduced ... (rest in TASKS.md)
- **002-T04** Runtime-local descriptor store with digest, TTL and probe budget. Status: CODE-COMPLETE. Evidence: CODE. Detail: CODE-COMPLETE with one deviation 2026-11-10 (`AgentDescriptorStore` in storage-core, implemented over the existing `agent_control_descriptors` table: append with compare-and-swap on the prior digest, latest/current reads, TTL boundary at expiry, oversize and non-object bodies refused; tests `descriptor_*` in ... (rest in TASKS.md)
- **002-T05** AgentControlDescriptor producer and served descriptor routes. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL (descriptor routes `getAgentControlDescriptor` and `refreshAgentProfileControl` now served for a profile whose provider selects the OpenCode module: bounded probe outside any transaction, budget `429 RATE_LIMITED`, `Cache-Control: no-store`, Codex/unknown profiles still `503`; route tests added. Not done: ... (rest in TASKS.md)
- **002-T06** Fresh-descriptor gate on binding enable and admission. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (enable gate enforced inside the enable transaction when the profile's module declares a descriptor: stale, missing or superseded revision fails with `DESCRIPTOR_STALE` mapped to `409 RESOURCE_STALE` and no write; exact replay returns the receipt; desktop enable reads or refreshes the descriptor and sends the revision. ... (rest in TASKS.md)
- **002-T07** Native sign-in begin and auth observation. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL 2026-11-10 (CODE + SERVED, fail-closed). `beginAgentAuthentication` is served by `agent_auth_operator.rs` and `AgentAuthService` (`agents/auth_service.rs`): it acts only on a method declared by the profile's fresh module descriptor, returns the in-flight challenge instead of a second launch, records ... (rest in TASKS.md)
- **002-T08** Replay-safe credential slot submission. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (retryable) 2026-11-10: the durable idempotency receipt needs a new additive table (a new migration; v31 and v32 are now taken by 014-T03 and 012-T05, so this is v33 per the allocation table in docs/LITECOWORK-BUILD-TASKS.md) and the matching `SecretStore` crash-reconcile path. Adding a migration touches the ... (rest in TASKS.md)
- **002-T09** Composer model, effort and declared native commands from the descriptor. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL 2026-11-10 (CODE, UI view-model only). `conversation-model-picker.ts` adds `deriveModelPicker`, `buildModelSelection` and `deriveCommandList`: descriptor-backed model list with selected check and provider-default marker, Effort row only when declared, distinct LOADING/ERROR/DENIED/OFFLINE/STALE/BLOCKED/EMPTY texts, ... (rest in TASKS.md)
- **002-T10** Supply 002 operations to the Feature 011 Settings -> Agents screen. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL 2026-11-10 (CODE + SERVED, fail-closed). `openAgentNativeConfiguration` is served by `agent_native_config_operator.rs`: the caller supplies only a `targetId`; an id with a path separator or control character is rejected, a target not declared in the fresh module descriptor is rejected before any opener is consulted, and ... (rest in TASKS.md)
- **002-T11** Catalog, inventory and honest install/update unavailability for V1. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL 2026-11-10 (CODE). Registry and inventory remain served; Install and Update are shown as unavailable with the reason "No qualified installer for this module in V1" and no Install or Update control exists to focus (test `install and update are visibly unavailable and no install control is offered`). Not done: the ... (rest in TASKS.md)
- **002-T12** Claude Code CLI as the second AgentModule (B-11). Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (retryable) 2026-11-10: the module needs the exact Claude Code CLI version, its non-interactive streaming transport and native auth method pinned from an installed CLI, and none is installed in this sandbox. Shipping a manifest with a guessed version pin would violate AC-017 and the T12 stop rule, so `CLAUDE_CODE` stays ... (rest in TASKS.md)
- **002-T13** Failure, restart and race recovery for probe, descriptor, auth and binding. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL 2026-11-10 (CODE). Added `agent_lifecycle_recovery_restart_revalidates_the_descriptor_until_reprobed` (a new RuntimeIncarnation never inherits a descriptor until re-probed) and `agent_lifecycle_recovery_failed_probe_leaves_the_prior_descriptor_intact` (crash or unpinned probe leaves the prior descriptor current and no ... (rest in TASKS.md)
- **002-T14** Cross-layer trace for every 002 user-visible operation. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL 2026-11-10 (DOCS). Trace table for Probe, Create/Enable/Disable binding, Refresh descriptor, Sign in, Select model and Open native config is in `contracts/ui-behavior.md` with real symbol names and test ids. Three breaks are recorded rather than hidden (no Tauri command for Sign in, Open native config or the picker; ... (rest in TASKS.md)
- **002-T15** Release-matrix qualification (CODE, SYSTEM, USER). Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED 2026-11-10 by owner instruction: release-matrix qualification (Windows x64 and Linux x64 packaged runs, owner USER evidence) depends on Windows hardware, the packaged installer and release items, and is to be tested later on the owner's laptop. No QUALIFIED claim is made anywhere in this package.
- **002-T16** Documentation and ledger sync. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL 2026-11-10 (DOCS). `served-route-inventory.json` regenerated (178 served / 1 local-only / 53 target-only; three audit scripts, `tests/test_architecture_validator.py`, IMPLEMENTATION-STATUS, the register, coverage and semantic maps updated), `docs/API.md` and `contracts/api-events.md` state the two newly served ... (rest in TASKS.md)
- **002-T17** Define the native-agent adapter as an ACP client (P-11). Status: PARTIAL. Evidence: CODE. Detail: PARTIAL 2026-11-10 (CODE, commit bf9b8e0). `agents/acp_client.rs` implements a bounded-framing ACP client (initialize, session/new, session/prompt, session/update, session/cancel, request_permission, fs and terminal routing to the Client owner, foreign-session traffic ignored, one cancel, fail-closed faults) with tests ... (rest in TASKS.md)
- **002-T18** Agent Matchmaker ranking for a job (P-12). Status: PARTIAL. Evidence: CODE. Detail: PARTIAL 2026-11-10 (CODE). `agents/matchmaker.rs` `rank_agents` ranks eligible bindings by capabilities declared in fresh descriptors, gives a plain-language reason per candidate, excludes disabled, not-lead-eligible, missing and stale-descriptor candidates with a reason and no rank, keeps caller order on ties and always ... (rest in TASKS.md)
- **002-T19** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED

### 003-workspace-roots-context-ingestion

Package status: ACCEPTED/LOCKED. Source: `specs/003-workspace-roots-context-ingestion/TASKS.md`.

- **003-T01** Promote accepted 003 contracts into owner docs and machine schemas. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **003-T02** Qualify root lifecycle storage on both rows. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (Linux, 2026-11-10: `workspace_root_lifecycle` tests in `crates/storage-sqlite/src/workspace_root_lifecycle_tests.rs` pass 3/3 covering pause hides root from scan candidates, stale-version loser, revoke deletes private bindings and is terminal (AC-005), idempotent replay, changed-payload refusal and foreign principal ... (rest in TASKS.md)
- **003-T03** Implement Windows WorkspaceRoot add on the GA row. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (Windows-only per owner instruction 2026-11-10; to be tested on the owner's Windows laptop later; nothing here is claimed done) - NOT STARTED (Windows fallback of `create_workspace_root_from_native_selection` in `apps/litecoworkd/src/operator.rs` returns 503; Tauri `add_workspace_root` already has a `#[cfg(windows)]` ... (rest in TASKS.md)
- **003-T04** Windows startup identity revalidation for saved roots. Status: Windows part DEFERRED. Evidence: CODE. Detail: Windows part DEFERRED (owner instruction 2026-11-10); Linux part not reached in this slice. PARTIAL (`list_workspace_root_revalidation_candidates` and `commit_workspace_root_revalidation` exist in `crates/storage-core/src/lib.rs`; Windows identity comparison through `RawDirectoryIdentity::matches_binding` in ... (rest in TASKS.md)
- **003-T05** Implement the Windows bounded manual rescan. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (Windows-only per owner instruction 2026-11-10; to be tested on the owner's Windows laptop later; nothing here is claimed done) - NOT STARTED (non-Linux returns 501 `SCAN_UNSUPPORTED_PLATFORM` at `scan_workspace_root`; Linux census in `scan_linux_root`; `windows_directory_entry_is_traversable` and ... (rest in TASKS.md)
- **003-T06** Symlink, reparse-point and path-identity denial matrix. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (Windows-only per owner instruction 2026-11-10; to be tested on the owner's Windows laptop later; nothing here is claimed done) - NOT STARTED
- **003-T07** Bounded content ingestion from ACTIVE roots (B-13). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (scan records metadata only; the index in `crates/storage-sqlite/src/resource_index.rs` covers uploaded Resources only)
- **003-T08** Platform-honest Persistent folders UI. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (Persistent folders section in `apps/litecowork-ui/src/App.tsx`, status labels in `apps/litecowork-ui/src/workspaces/workspace-root-status.ts`, scan unsupported handling in `workspace-root-scan.ts`; Add is not disabled before selection on Windows; no post-picker confirm)
- **003-T09** ContextAttachment and retrieval-receipt stores. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (no table, store trait or event exists; `CONTEXT_ATTACHMENT` appears only as an AgentControl capability enum)
- **003-T10** Call retrieval from Send and commit the receipt before dispatch. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (`rag_retrieval_operator.rs` is not called by `conversation_operator.rs`)
- **003-T11** @-reference ResourceRef picker in the Feature 001 composer. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (placeholder "@ to mention" and an "Add context or file" button with no handler in `apps/litecowork-ui/src/conversations/ConversationsPage.tsx`; submit sends `resourceRefs: []`)
- **003-T12** Context Used surface for grounded answers. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (`apps/litecowork-ui/src/resources/retrieval-grounding.ts` decoder exists with no production call site)
- **003-T13** Failure, race and recovery suite. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **003-T14** Cross-layer traces for every 003 user operation. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **003-T15** Qualify E2E-03 on the Windows and Linux release rows. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **003-T16** Documentation and ledger sync. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **003-T17** Context Debugger surface (P-15). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - added by owner approval on 2026-11-09 (docs/OWNER-ACCEPTANCE-REVIEW-PACKET.md). Folds proposed task P-15 from docs/REFERENCE-SYNTHESIS-AND-V1-FEATURE-ALIGNMENT-2026-11.md section 8 into this LOCKED package.
- **003-T18** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED

### 004-agent-relay

Package status: ACCEPTED/LOCKED. Source: `specs/004-agent-relay/TASKS.md`.

- **004-T01** Promote portable-context bounds, truncation receipt and relay errors. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **004-T02** Preserve producer identity through the desktop message types. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (Operator and OpenAPI `ConversationMessage` carry nullable `agent_binding_id` and `agent_session_id`; the Tauri bridge forwards the JSON unchanged; `parseConversationSnapshot` drops both).
- **004-T03** Portable relay export with filtering, bounds and truncation receipt. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (`build_conversation_context_package` orders by event revision, excludes `SYSTEM_NOTICE`, carries `source_agent_binding_id`, enforces 1,024 messages / 4 MiB; no truncation receipt, no cross-Workspace reference check, no adapter-budget input).
- **004-T04** Relay admission: fresh revalidation, no fallback, single-flight, no Task writes. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (`setConversationAgentBinding` commits future-turn selection with CAS and idempotency; admission checks a fresh compatible offer through `resolve_conversation_session_host_in_transaction`; single-flight and the descriptor gate are not enforced).
- **004-T05** Claude Code dispatch arm in AgentSessionSupervisor (B-16). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (`execute_agent_turn` has one `"opencode"` arm and an `other =>` unsupported arm).
- **004-T06** Picker gated on dispatchability and descriptor freshness. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (`buildConversationAgentChoices` filters Workspace, enabled and lead-eligible; `deriveConversationAgentSurface` exists with tests and no production caller).
- **004-T07** Relay UI states and producer label. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **004-T08** Relay failure and recovery. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **004-T09** Cross-layer trace for relay operations. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **004-T10** E2E-02 qualification on the release matrix. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **004-T11** Documentation and ledger sync. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **004-T12** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED

### 005-safe-task-path

Package status: ACCEPTED/LOCKED. Source: `specs/005-safe-task-path/TASKS.md`.

- **005-T01** Promote accepted 005 contracts into owner docs and machine schemas. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **005-T02** Consolidate Environment types and provider probes into the owning crates (Rust). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (duplicates confirmed in source, see Current baseline)
- **005-T03** Plan acceptance domain and storage with CAS and idempotency. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (`accept_initial_plan` / `PlanAcceptanceCommit` / `accept_initial_plan_transaction` exist with tests in `crates/storage-core/src/lib.rs` and `crates/storage-sqlite/src/lib.rs`; `parse_initial_plan_output` in `crates/domain-task/src/planning.rs`; no producer-scope check against a live planning session because sessions ... (rest in TASKS.md)
- **005-T04** Linux bubblewrap Environment provider with four ENFORCED, attested dimensions. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (launch primitive `spawn_bubblewrapped` / `prepare_bubblewrap_command` and live tests `live_bubblewrap_exposes_pinned_inputs_read_only_and_only_output_writable`, `live_bubblewrap_network_namespace_cannot_reach_host_loopback` exist in `crates/linux-process-scope/src/bubblewrap.rs`; `build_attestation`, ... (rest in TASKS.md)
- **005-T05** Native lead planning dispatch behind the isolation gate (Linux row). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (`require_dispatch()` always errors; `dispatch_available=false` hardcoded in `apps/litecoworkd/src/agents/planning.rs`; `require_task_planning_isolation_admission()` closes storage)
- **005-T06** B-18 atomic whole-file replacement primitive (Rust). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (`replace_file_write_through` exists for Windows in `crates/windows-local-security/src/lib.rs`; Linux path confinement helpers exist in `bubblewrap.rs` `validate_input_tree`; no write primitive)
- **005-T07** Trust wiring, file-write grant and exact Effect approval. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL (`apps/litecoworkd/src/trust.rs` evaluator and approval producer exist under `#[allow(dead_code)]`; `trust_operator.rs` `create_read_grant` serves only `workspace.resource.read`; approval resolve with exact `confirmation_digest` is served via `approval_operator.rs`)
- **005-T08** Attempt admission service: attestation gate, lease, grant, receipt, then Environment call. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (`StepAttemptCoordinator::admit`, `decide_attempt_admission`, SQLite `admit_step_attempt` exist with no daemon caller)
- **005-T09** Effect reconciliation and independent verification. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (`EffectRecord::transition` / `authorize_retry`, `EvidenceRecord` in `crates/domain-effects/src/lib.rs`; `transition_effect` / `append_evidence` in `crates/storage-sqlite/src/effect_evidence.rs`; `verification_runs_operator.rs` read-only; no reconciler or verifier caller)
- **005-T10** Operator routes and Tauri bridge commands for plan, Attempt, Effect, cancel and recover. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (target-only in `docs/schemas/served-route-inventory.json`: `submitTaskPlan`, Attempt list/admit, Effect list, `cancelTask`, `recoverStep`)
- **005-T11** Cancel, quiescence, crash and restart recovery. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (`evaluate_quiescence`, `QuiescenceObservation`, `ManagedAttemptScope::wait_for_quiescence`, `AttemptProcessScopeJournal::incomplete_records` exist as dead code; `decide_lease_expiry` exists in `crates/domain-task/src/execution.rs`)
- **005-T12** Conversation to Task promotion (storage, route and UI). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (`validate_task_create_commit` refuses Conversation origin; `ConversationsPage.tsx` has no control; Home "New task" works for standalone objectives)
- **005-T13** Work screen execution UI with full state coverage. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (`TaskDetailPanel` in `apps/litecowork-ui/src/App.tsx` shows read-only planning readiness via `apps/litecowork-ui/src/tasks/planning-readiness.ts`, Steps, `TaskPresentationPanel` and outputs; no execution controls)
- **005-T14** Windows row: Task execution visibly UNAVAILABLE until B-35 qualifies. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (`probe_environment_provider` in `apps/litecoworkd/src/environment_provider.rs` reports a Windows LocalWorkspace class with the note "Job Object process-tree isolation and KILL_ON_CLOSE active", which reads as an isolation claim)
- **005-T15** Linux-row qualification: E2E-04 with cross-layer traces. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **005-T16** Owner decision on B-35 Windows Environment provider. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (B-35 PROPOSED; owner: repository owner, with Environments and Security)
- **005-T17** Windows Environment provider: AppContainer, restricted token, Job Object, WFP egress (Rust). Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (needs T16 acceptance; `spawn_kill_on_drop_job` and `replace_file_write_through` exist in `crates/windows-local-security/src/lib.rs`, nothing else)
- **005-T18** Windows-row qualification of Task execution (E2E-04). Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (needs T17; B-35 pending owner)
- **005-T19** Documentation and ledger sync. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **005-T20** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED

### 006-golden-connection-capability-write

Package status: ACCEPTED/LOCKED. Source: `specs/006-golden-connection-capability-write/TASKS.md`.

- **006-T01** Promote accepted 006 contracts into owner docs and machine schemas. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **006-T02** Owner decision on the GitHub OAuth recipe (client-secret finding). Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (GitHub docs require `client_secret` for web-flow exchange and revocation; B-19 stop clause applies; owner: repository owner with Connectors and Security)
- **006-T03** OS-keystore secret custody with a broker-only resolver and delete (Rust). Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (`OsSecretStore::write` and `keyring` backends exist; no resolve or delete on the public trait)
- **006-T04** Brokered HTTP egress policy: allowlist, TLS, SSRF, redirects, bounds, redaction (Rust). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (`reqwest` with `rustls-tls` is a dependency; no egress policy module)
- **006-T05** ConnectionService: AuthAttempt, OAuth flow per T02, Connection commit after keystore write. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (needs T02 recipe decision; no ConnectionService exists)
- **006-T06** Token refresh with per-connection lock and state mapping. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **006-T07** Open Invocation creation and DISPATCHED through the combined admission transaction only. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (`admit_capability_dispatch` implemented and tested; `sqlite-v15.sql` triggers keep it closed)
- **006-T08** Golden read through the broker with an explicit connector read grant. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **006-T09** Golden write: create issue comment with exact approval, bounds and self-approval denial. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **006-T10** UNKNOWN reconciliation for the comment write. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (`reconcile_ambiguous_terminal` in `crates/domain-invocations/src/lib.rs`; AMBIGUOUS transitions allowed by the v15 guard; no reconciler)
- **006-T11** Revoke and disconnect with local fencing and explicit remote-revoke state. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **006-T12** Operator routes, authorization and Tauri bridge commands. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (`listConnections`, `getConnection`, `disconnectConnection` target-only; `connect_or_install_capability` 503)
- **006-T13** Discover Connections UI, connection detail and golden operation flow. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (`DiscoverPage.tsx` Connections tab shows "Connections cannot be read here")
- **006-T14** Crash and restart recovery for AuthAttempts, orphan secrets and in-flight Invocations. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **006-T15** Qualification: E2E-05 on the Windows row, then the Linux row, with cross-layer traces. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **006-T16** Documentation and ledger sync. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **006-T17** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED

### 007-professional-artifacts

Package status: ACCEPTED/LOCKED. Source: `specs/007-professional-artifacts/TASKS.md`.

- **007-T01** Promote accepted 007 contracts into owner docs and machine schemas. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **007-T02** Atomic Artifact create store (B-30). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (no create method in `crates/storage-core/src/artifact.rs`; tests seed Artifact rows directly)
- **007-T03** Markdown format adapter (B-21). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **007-T04** Markdown version publish with CAS. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (`append_text_version` and `ArtifactVersionWriteStore::append_artifact_version` publish `text/plain` with `If-Match`, content version, Resource version and parent ResourceRevision pins)
- **007-T05** Declared format capability matrix producer. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (capability is inferred in the UI: `supportsTextPreview` in `artifact-api.ts` and the `text/plain` edit gate in `ArtifactWorkbench.tsx`)
- **007-T06** Export command and ArtifactExportReceipt (B-31). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED. The Product gate is closed: destination class `LOCAL_OWNER_SELECTED_FILE` confirmed 2026-11-10. Still depends on T01, T03 and T05. No export record, route or event exists.
- **007-T07** Operator routes and Tauri bridge commands. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **007-T08** Conversation association of Artifacts. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (`ArtifactRecord` has `task_id` only; no Conversation producer)
- **007-T09** Library and Workbench create, Markdown edit and export UI. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (`ArtifactLibrary.tsx` lists and opens; `ArtifactWorkbench.tsx` edits `text/plain`, compares, promotes, archives and Saves As; no create, no Markdown edit, no export receipt history)
- **007-T10** DOCX, XLSX, PPTX and PDF read-only or unsupported rendering. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (Library uploads keep these as Resources with preview/no-preview or Save original; there is no declared matrix and no explicit unsupported card in `ArtifactWorkbench.tsx`)
- **007-T11** Stored-version compare. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (`TextVersionDiff.tsx` with `lineDiff` in `text-version-diff-model.ts`, bounded to 400 lines and 160,000 cells, comparing loaded stored versions)
- **007-T12** Imported and Task-generated source classes. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED for the Task-generated class (Feature 005 produces no Task output today; owner: Task); NOT STARTED for Imported
- **007-T13** Failure, restart and reopen suite. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **007-T14** Cross-layer traces for every 007 user operation. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **007-T15** Qualify E2E-07 on the Windows and Linux release rows. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **007-T16** Documentation and ledger sync. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **007-T17** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED

### 008-workbench-browser-terminal-computer

Package status: ACCEPTED/LOCKED. Source: `specs/008-workbench-browser-terminal-computer/TASKS.md`.

- **008-T01** Promote accepted Workbench contracts into owner docs and schemas. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **008-T02** Finish Workbench truthfulness: Activity fallthrough, Computer UNAVAILABLE, per-row reasons. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (register A-51 done: `TerminalPane`, `ReviewPane` and empty Browser default in `apps/litecowork-ui/src/workbench/WorkbenchPanel.tsx`, guarded by `apps/litecowork-ui/tests/workbench-truthfulness.test.mjs`; remaining: `activity` falls through to `ReviewPane`, no Computer status, no platform-specific terminal reason)
- **008-T03** Browser session, lease and receipt storage with CAS and idempotency. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **008-T04** Managed Chromium provider with isolated profile (B-22). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **008-T05** BrowserSessionService, Operator routes and Tauri bridge. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **008-T06** Browser Workbench view with consent, take control and return. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **008-T07** Browser failure and recovery: provider loss, restart, races, replay. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **008-T08** Terminal session and ordered receipt storage. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **008-T09** Safe shell discovery and Linux PTY under an admitted bubblewrap Environment. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (needs an admitted Linux bubblewrap Environment from Feature 005, owner Environments)
- **008-T10** TerminalSessionService, Operator routes, stream bridge and backpressure. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **008-T11** Terminal Workbench view with per-row availability. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **008-T12** Windows ConPTY provider under a qualified Windows Environment. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (Feature 005 blocker B-35, Windows Environment provider, PROPOSED, owner Environments/Security)
- **008-T13** Terminal failure and recovery: stop, close, crash, restart, races. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **008-T14** Workbench Activity view from committed records. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **008-T15** Exact-revision and VCS Diffs. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (an unrelated bounded LCS `lineDiff` exists in `apps/litecowork-ui/src/artifacts/text-version-diff-model.ts` for Artifact comparison)
- **008-T16** Cross-layer traces for every Workbench user operation. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **008-T17** Qualify E2E-08 on each release-matrix row. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **008-T18** Documentation sync. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **008-T19** Remove allow-same-origin from the Browser preview iframe and add a guard test (P-06). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - added by owner approval on 2026-11-09 (docs/OWNER-ACCEPTANCE-REVIEW-PACKET.md). Folds proposed task P-06 from docs/UI-RENDERING-SETTINGS-ARCHITECTURE-AUDIT-2026-11.md section 9 (finding U-02) into this LOCKED package.
- **008-T20** Git worktree view, VCS diff and user-invoked GitHub PR ops (P-13). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - added by owner approval on 2026-11-09. Folds proposed task P-13 from docs/REFERENCE-SYNTHESIS-AND-V1-FEATURE-ALIGNMENT-2026-11.md section 8 into this LOCKED package.
- **008-T21** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED

### 009-coworkers-scoped-memory

Package status: ACCEPTED/LOCKED. Source: `specs/009-coworkers-scoped-memory/TASKS.md`.

- **009-T01** Promote accepted Coworker contracts into owner docs and schemas. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - owner acceptance of B-24 and B-25 recorded 2026-11-09 (SPEC revision 1.0). No accepted interface is promoted: no Coworker Conversation route, assignment, responsibility proposal or Coworker origin column exists in `docs/schemas/`.
- **009-T02** Store an immutable Coworker origin on Conversations. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED. `ConversationRecord` and the `conversations` table have no Coworker field.
- **009-T03** Serve Coworker Conversation create/list through the Operator and Tauri bridge. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **009-T04** Re-widen the ContextDocument create enum to accept COWORKER_NOTES (A-18/B-24 rider). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED. The narrowed enum and the two-kind validator are in source today.
- **009-T05** Serve owner-authored Add Memory as a Coworker-scoped ContextDocument (B-24). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED. No Coworker memory path exists; `coworker_learning.rs` has no caller.
- **009-T06** Revoke and purge Coworker memory with fences and receipts. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL - ContextDocument status and deletion routes (`/v1/resources/{resource_id}/context-document/status`, `/deletion`) and `crates/storage-sqlite/src/context_document_purge.rs` exist for the two served kinds, with tests in `context_document_purge_tests.rs`. They are not exercised for COWORKER_NOTES and retrieval does not ... (rest in TASKS.md)
- **009-T07** Coworker-scoped retrieval that enforces CoworkerContextPolicy. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED. Retrieval has no Coworker filter; `CoworkerContextPolicy` has no consumer.
- **009-T08** Quick Create opens an UNBOUND first Coworker chat (B-25). Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - `HomeCoworkerOnboarding.tsx` creates the Coworker (and can set primary) with `tests/home-onboarding.test.ts`; it opens no chat and does not show the unbound state.
- **009-T09** Coworker chats and Memory & Knowledge UI. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED. `CoworkerSettings.tsx` has roster, detail, revision form, status actions and presence (503) but no chats list or memory view.
- **009-T10** Coworker pause fences enabled Automations and resume reconciles them. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - admission rejects a non-ACTIVE Coworker (`validate_coworker_provenance`) and enable checks ACTIVE, but the TriggerHost turns that into a 30 s retry loop, `resume_reconciled: !has_active_automation` blocks resume while an Automation is enabled, `check_origin_status` has no caller, and the Home composer rejects PAUSED ... (rest in TASKS.md)
- **009-T11** Responsibilities as proposals that become admitted Routines, with no grant. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED. Routine and Automation aggregates exist (`apps/litecoworkd/src/routine_operator.rs`, `automation_operator.rs`); no responsibility record or route exists.
- **009-T12** Coworker capability assignment as a ceiling. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED. No assignment record or route exists.
- **009-T13** Failure and recovery for Coworker chat, memory and pause. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **009-T14** Cross-Workspace leakage fixture (CODE and owner USER). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **009-T15** Cross-layer trace for each Coworker operation. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **009-T16** Qualify E2E-09 on the release matrix rows. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED - needs Feature 001 Send and the Feature 002 OpenCode binding on Windows x64 (owners: Conversation, Agent Runtime); Linux x64 row second.
- **009-T17** Synchronize status documents and inventories. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **009-T18** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.

### 010-automations-recurring-host

Package status: ACCEPTED/LOCKED. Source: `specs/010-automations-recurring-host/TASKS.md`.

- **010-T01** Promote accepted Automation and notification contracts into owner docs and schemas. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - owner acceptance of B-26, B-27 and A-35 recorded 2026-11-09 (SPEC revision 1.0). Nothing promoted: the cron grammar and catch-up bound are not pinned in AUTOMATION.md, delivery transitions have no route or schema, and `COWORKER_PAUSED` is not a slot outcome token.
- **010-T02** Remove stale hosting claims from the Operator header and Pause copy (A-29). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED. The stale text is at `automation_operator.rs` lines 1-5 and `AutomationsPage.tsx` line 251.
- **010-T03** Qualify cron in a named timezone with the accepted misfire policy (B-26). Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - `ScheduleTrigger` parsing, DST rules and misfire handling exist with tests in `crates/domain-responsibility/src/schedule_tests.rs` and `crates/storage-sqlite/src/automation_host_tests.rs` (`a_cron_schedule_trigger_parses_into_an_executable_plan`, ... (rest in TASKS.md)
- **010-T04** Harden occurrence identity, dedupe and conflict. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - `occurrence_for_slot` returns the original Task for a repeated slot and `a_reused_task_request_key_with_a_changed_payload_is_a_conflict` exists. Concurrent settlement from two passes and `OCCURRENCE_CONFLICT` for a stable delivery id with a new digest are not proven for every trigger kind.
- **010-T05** Recheck Coworker PAUSED at occurrence settlement. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - admission refuses a non-ACTIVE Coworker (`validate_coworker_provenance`), but `settle_one_trigger` turns that into a blocker and retries the slot every pass; no skip is recorded and the cursor does not advance.
- **010-T06** Deliver LOCAL_POLL notifications with status separate from Task outcome (B-27). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED. `PENDING` rows are inserted; nothing claims, presents or settles them.
- **010-T07** Record SUPPRESSED deliveries and serve the quiet-hours control surface (A-35). Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - `QuietHours` parsing and `decide_notification` suppression exist; `set_notification_preference` accepts `quiet_hours` and `valid_quiet_hours` checks it. Suppression writes no row (contrary to AC-007) and `NotificationsPage.tsx` has no control to change quiet hours.
- **010-T08** Hand occurrence Tasks to execution only through Feature 005. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED - Feature 005 Task execution is not qualified (owner: Task Runtime). Occurrences create READY Tasks and `settle_started_occurrences_from_task_outcomes` already settles occurrences from terminal Task outcomes.
- **010-T09** Add the event-trigger kind after cron (CONNECTOR_EVENT over LOCAL_POLL). Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED - the kind is owner-confirmed (CONNECTOR_EVENT over LOCAL_POLL, sarv-projects, 2026-11-10); still needs a qualified Feature 006 Connection (owner: Capability Fabric). `TriggerDefinition::ConnectorEvent` exists; `locally_reconcilable_trigger` refuses it.
- **010-T10** Window close, Runtime stop and sleep matrix. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED. The worker runs while the daemon runs; no test covers window close versus Runtime stop or OS sleep.
- **010-T11** Automation history and editor UI states. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - `AutomationsPage.tsx` lists definitions with Load more, shows health and the first occurrence page; `AutomationEditor.tsx` edits triggers. History has no pagination and no delivery status column.
- **010-T12** Failure and recovery: crash, replay, lease expiry and races. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - cursor monotonicity and repeated settlement are tested in `automation_host_tests.rs`; crash between occurrence commit and Task, CLAIMED lease expiry and restart during delivery are not.
- **010-T13** Security negatives: no grant, no inbound port, untrusted payloads. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - Routine run rejects bindings beyond 64 and grants nothing (`apps/litecoworkd/src/routine_run_operator_tests.rs`); no socket inventory or event-payload containment test exists.
- **010-T14** Cross-layer trace for each Automation operation. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **010-T15** Qualify E2E-10 (scheduled and event Automation) on the release matrix rows. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED - the event half needs T09 (Feature 006 Connection); Task execution needs Feature 005 on the Linux row and the B-35 Windows Environment provider on the Windows row (owner: Task Runtime).
- **010-T16** Synchronize status documents and inventories. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **010-T17** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **010-T18** Qualified external WorkflowBackend Routine mapping (not a V1 release gate; formerly T05). Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED - depends on qualified Feature 005 Task admission, Feature 006 CapabilityInvocation and a WorkflowBackendAdapter, none present (owners: Task Runtime, Capability Fabric).
- **010-T19** Direct LiteCowork Coworker provider-graph authoring evaluation (deferred, not V1; formerly T06). Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED - deferred; the owner confirmed on 2026-11-10 that it stays deferred and out of V1; no implementation authorization until an owner ADR exists.

### 011-desktop-shell-ipc-settings-runtime

Package status: ACCEPTED/LOCKED. Source: `specs/011-desktop-shell-ipc-settings-runtime/TASKS.md`.

- **011-T01** Promote accepted 011 interfaces into owner docs and schemas. Status: DONE. Evidence: CODE. Detail: DONE (CODE, contract level): `RuntimeStopPreview` gained settlement lists and `verdict`; `requestLocalRuntimeStop` gained `If-Match`; `PowerAssertion` added as optional `RuntimeLifecycleView.power_assertion`; `renameWorkspace` (PUT /workspaces/{workspaceId}/name) added target-only with `workspace.renamed.v1`; error codes ... (rest in TASKS.md)
- **011-T02** Qualify the Windows named-pipe DACL profile (B-28 first row). Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (needs a native Windows 11 or 10 x64 runner; WSL2 does not count). Prior partial state kept: PARTIAL (`crates/operator-ipc/src/windows.rs` endpoint/listener and `crates/windows-local-security/src/lib.rs` `create_operator_pipe`, `current_logon_sid` exist with unit tests; missing: DACL-failure no-fallback, remote-client ... (rest in TASKS.md)
- **011-T03** Qualify the Linux 0600/0700 socket profile (second row). Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED to the second-user Linux qualification run (needs a second OS user and the packaged build). Prior partial state kept: PARTIAL (`crates/operator-ipc/src/unix.rs` enforces directory/socket modes and peer credentials; `crates/operator-ipc/tests/unix_endpoint_lifecycle.rs` exists; missing: native Linux desktop ... (rest in TASKS.md)
- **011-T04** IPC admission bounds and frame isolation. Status: DONE. Evidence: CODE. Detail: DONE (CODE, Linux unix transport; Windows transport shares the same `InFlightBodyBudget` and eight-slot semaphore but the Windows concurrency test is deferred to a native Windows runner, see T02). `crates/operator-ipc/tests/ipc_admission_bounds.rs` added; `connection_admission_is_bounded_and_releases_capacity_after_disconnect` ... (rest in TASKS.md)
- **011-T05** Runtime health honesty in the shell. Status: DONE. Evidence: CODE. Detail: DONE for the UI (CODE): `src/runtime/runtime-health.ts` derives transport, Operator, recovery_state and execution rows from observations; `SettingsView.tsx` renders loading/error states in live regions and no longer says "Localhost"; `tests/settings-runtime-health.test.mjs` added (5 tests). The Tauri-side `cargo test -p ... (rest in TASKS.md)
- **011-T06** Remove inert Settings controls and gate by owner class (B-29/B-33). Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (CODE): `settings-control-map.ts` maps all 19 controls rendered directly by `SettingsView.tsx` (each carries `data-control-id`); `tests/settings-inert-control-audit.test.mjs` (5 tests) fails on an unmapped control, a Target-class control or a missing unavailable entry; shortcut card removed; Usage, Storage and About ... (rest in TASKS.md)
- **011-T07** Applied and persisted local display preferences. Status: DONE. Evidence: CODE. Detail: DONE (CODE): `src/settings/local-preferences.ts` validates, persists (webview localStorage under `litecowork.local-preferences.v1`) and applies theme, density, text scale and reduced motion at the document root from `App.tsx`; `tests/local-preferences.test.mjs` (7 tests). Restart survival is proven by reloading from the same ... (rest in TASKS.md)
- **011-T08** Workspace rename and archive in Settings. Status: DONE. Evidence: CODE + SERVED. Detail: DONE for Linux (CODE, SERVED): `WorkspaceService::rename_idempotent` (domain-workspace) over the existing `commit_workspace_idempotent` transaction (no migration), Operator `PUT /v1/workspaces/{workspace_id}/name` (`rename_workspace` in `operator.rs`), `rename_workspace` Tauri command, `workspace-rename.ts` and an inline rename ... (rest in TASKS.md)
- **011-T09** Settings -> Agents screen over served Feature 002 operations (A-35). Status: DONE. Evidence: CODE. Detail: DONE (CODE). Agents heading derives from probed profiles (`src/settings/agents-heading.ts`, 4 tests); descriptor 503 renders UNAVAILABLE via `AgentControlDescriptorPanel`; all 35 interactive elements in `AgentCatalogSettings.tsx` carry `data-control-id` and are classified in `AGENT_CATALOG_CONTROL_MAP` ... (rest in TASKS.md)
- **011-T10** Notifications entry linking Feature 010 quiet hours. Status: DONE. Evidence: CODE. Detail: DONE (CODE): Notifications category links to the Notifications page and states that quiet-hours editing is unavailable until Feature 010; no quiet-hours control exists in this package; `settings-inert-control-audit.test.mjs` pins it
- **011-T11** App-wide keyboard and focus baseline. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL (CODE): `src/shell/shortcut-registry.ts` registers Ctrl N and Ctrl K with typed handlers; skip link (`Skip to main content`) targets `<main id="main-content" tabIndex={-1}>`; one global `:focus-visible` outline now covers button, a, textarea, input, select, summary and `[tabindex]`; the Settings General category lists ... (rest in TASKS.md)
- **011-T12** Keep-awake power assertion. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL (CODE, not wired). The prerequisite definition of "work runs" now exists (admitted Conversation turns and non-terminal Attempts from `RuntimeLifecycleStore::read_runtime_stop_observation`, T13 prerequisite). DONE: `apps/litecoworkd/src/power_assertion.rs` state machine (HELD only after the backend confirms; an OS ... (rest in TASKS.md)
- **011-T13** Authenticated Operator stop with settlement proof. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL (prerequisite half DONE, request half BLOCKED). DONE (CODE, SERVED, Linux): the read-only `previewLocalRuntimeStop` (`GET /v1/runtime-lifecycle/stop-preview`) over `RuntimeLifecycleStore::read_runtime_stop_observation` (admitted Conversation turns, non-terminal Attempts, unresolved Effects ... (rest in TASKS.md)
- **011-T14** Desktop stop and update UI with quiescence gate. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL (CODE; Tauri bridge not compiled here). DONE: `src/runtime/runtime-stop-state.ts` (pure summary: Stop safely disabled with reasons for BLOCKED/UNKNOWN and always while the stop command is unmounted, update gate allowed only on QUIESCENT for the exact incarnation, `runtimeStoppedLabel` shows "Runtime stopped" only from ... (rest in TASKS.md)
- **011-T15** Shell failure and recovery. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (`apps/litecoworkd/src/runtime_recovery_tests.rs`, `runtime_shutdown_tests.rs`, `runtime_process_tests.rs` cover daemon-side recovery and drain; missing: desktop-side disconnected state, window-close vs Runtime state, simultaneous stop/update)
- **011-T16** Cross-layer traces for shell and Settings operations. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL. The trace table is in `contracts/ui-behavior.md` ("Cross-layer traces (T16)") with symbols checked against source on 2026-11-10; it lists three gaps instead of documenting them as present (keep-awake wiring and UI, Stop safely, update check and install commands). The Settings controls of SettingsView and ... (rest in TASKS.md)
- **011-T17** Packaged qualification on Windows x64 then Linux x64. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (packaged qualification needs native Windows x64 then Linux x64 hosts and the packaged installers from Feature 019). NOT STARTED
- **011-T18** Documentation sync. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL: API.md, RUNTIME-LIFECYCLE.md, V1-SETTINGS-CONTRACT-LEDGER (generated), SEMANTIC-SERVED-UI-MAP, FEATURE-AND-UI-CONTRACT-COVERAGE, the contradiction register A-03 and IMPLEMENTATION-STATUS were updated for the served stop preview and 175/1/55; the ui-behavior trace table was added. NOT done: the V1 UI action-state ledger ... (rest in TASKS.md)
- **011-T19** Shell reachability and truthfulness (P-05). Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (CODE). CORRECTION to the earlier claim: `navigation` was a dead array that the sidebar did not render, so Goals, Notifications, Ideas and Home had no navigation path and the earlier reachability test passed falsely. Now the sidebar renders a "More" group from `navigation` (Home as "New task", Goals, Notifications, ... (rest in TASKS.md)
- **011-T20** Decompose App.tsx and the Rust god files behind existing boundaries (P-07 and P-19). Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (UI slice only; no behavior change; all UI tests unchanged except two source-string pins updated in earlier commits). DONE: one shared `src/shell/OperatorUnavailablePanel.tsx` replaces the six duplicated "Local Operator unavailable" page bodies (Coworkers, Notifications, Ideas, Routines, Discover, Automations); ... (rest in TASKS.md)
- **011-T21** Merge SettingsPage into SettingsView with precedence notes (P-08). Status: DONE. Evidence: CODE. Detail: DONE (CODE). `SettingsPage`, `SettingsPageProps` and `localServiceCopy` are deleted from `App.tsx`; `SettingsView` is the one Settings authority (guarded by `tests/settings-merge.test.mjs`, 4 tests). Merged into `SettingsView`: instruction-conflict recovery ("Use this version", `workspaces-instructions-use-latest`), revision ... (rest in TASKS.md)
- **011-T22** Persistent Workspace Dashboard with pinnable widgets (P-16). Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL (CODE). `src/dashboard/dashboard-state.ts` (pure: per-Workspace pins in localStorage `litecowork.dashboard.pins.v1`, validated, corrupt state falls back to all pinned, bounded workspace entries, count summaries that flag first-page truncation), `WorkspaceDashboard.tsx` (pin/unpin with `aria-pressed`, UNAVAILABLE with ... (rest in TASKS.md)
- **011-T23** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED

### 012-conversation-coedit-take-control

Package status: ACCEPTED/LOCKED. Source: `specs/012-conversation-coedit-take-control/TASKS.md`.

- **012-T01** Record owner acceptance and promote the accepted co-edit contracts. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - B-34 owner acceptance (scoped) recorded 2026-11-09 in this revision with the RV-14 no-vendor-claim constraint; no interface is promoted into owner docs or docs/schemas.
- **012-T05** Add control-event and side-note storage with CAS and idempotency. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - storage layer done (2026-11-10): `docs/schemas/sqlite-v32.sql` (`conversation_control_events` append-only, `conversation_owner_side_notes`), `ConversationCoeditStore::{append_owner_side_note, list_owner_side_notes}`, and `SettleConversationTurn.control_event` which appends `OWNER_TOOK_CONTROL` in the same transaction ... (rest in TASKS.md)
- **012-T02** Implement take-control over a running turn. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - unblocked: Feature 001 T07 (effective Stop with observed quiescence) and T11 are DONE and the T05 storage layer has landed; not reached in this slice. Original blocker text: waits on Feature 001 effective Stop with observed quiescence (A-25; owner Agent Runtime).
- **012-T03** Implement correlated side-chat and exclude it from the native prompt. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - unblocked: Feature 001 T07 (effective Stop with observed quiescence) and T11 are DONE and the T05 storage layer has landed; not reached in this slice. Original blocker text: waits on Feature 001 effective Stop/quiescence (B-34 caveat; owner Agent Runtime).
- **012-T04** Project and render the truthful driver indicator. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - unblocked: Feature 001 T07 (effective Stop with observed quiescence) and T11 are DONE and the T05 storage layer has landed; not reached in this slice. Original blocker text: depends on T02 and T03 (and therefore on Feature 001 Stop/quiescence).
- **012-T06** Add Tauri commands and the take-control and side-chat composer controls. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - unblocked: Feature 001 T07 (effective Stop with observed quiescence) and T11 are DONE and the T05 storage layer has landed; not reached in this slice. Original blocker text: depends on T02-T04.
- **012-T07** Prove crash, restart, replay and timeout recovery. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - unblocked: Feature 001 T07 (effective Stop with observed quiescence) and T11 are DONE and the T05 storage layer has landed; not reached in this slice. Original blocker text: depends on T02 and T03.
- **012-T08** Record the cross-layer trace for take-control and side-chat. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **012-T09** Qualify co-edit on the release matrix. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **012-T10** Synchronize status documents and inventories. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **012-T11** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.

### 013-litespm-package-lifecycle-bridge

Package status: IN_REVIEW. Source: `specs/013-litespm-package-lifecycle-bridge/TASKS.md`.

- **013-T01** Owner acceptance of SPEC and blockers. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED - B-36 and B-37 are PROPOSED; pending owner acceptance (repository owner, sarv-projects).
- **013-T02** Re-pin LiteSPM and record the bridge contract baseline (lane steps 0-1). Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01; upstream publication of descriptor/approval/operation-get, LiteSPM owner).
- **013-T03** Promote accepted interfaces into owner docs and schemas. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01).
- **013-T04** LiteSpmClient: transport, handshake, version negotiation and peer authentication. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01). Nothing exists; reusable framing in `crates/operator-ipc`, SID helper in `crates/windows-local-security`.
- **013-T05** Host compatibility profile and advertised-operation gating. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01; B-37).
- **013-T06** Live inventory read-through and Discover projection. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01). PARTIAL foundation: Discover search/describe and `DiscoverProviderStatus` exist in `crates/storage-sqlite/src/discover.rs`.
- **013-T07** Approval and operation-receipt storage with CAS and idempotency. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01).
- **013-T08** Plan and approval routes with owner-only authorization. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01).
- **013-T09** Apply, receipt, read-back and UNKNOWN reconciliation with startup sweep. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01).
- **013-T10** Adopt and unmanage (observe-first). Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01; upstream unmanage absent at `f77a3a3`, O-06).
- **013-T11** Rollback, remove, update and repair. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01).
- **013-T12** Tauri package lifecycle bridge commands. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01).
- **013-T13** Discover UI: Installed / native, Managed, plan review and lifecycle verbs. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01). PARTIAL foundation: tabs and truthful unavailable states in `DiscoverPage.tsx`; `connectActionOutcome()` in `discover-state.ts`.
- **013-T14** Capability bridge input producer and Feature 006 handoff. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01). PARTIAL foundation: `classify_agent_capability_route` exists with tests and no caller.
- **013-T15** Discover truthfulness fixes (timestamps, fail-closed connect-or-install). Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01). PARTIAL: fail-closed 503 already correct; timestamp defect present in `discover_operator.rs`.
- **013-T16** Failure, crash and race matrix; create the canonical test target. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01).
- **013-T17** Cross-layer trace for every user-visible verb (AGENTS.md section 4). Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01).
- **013-T18** Qualification on the release rows (E2E-06, LSI rows). Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01; pinned LiteSPM build available on the release machines).
- **013-T19** Documentation sync. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01).
- **013-T20** Pin the LiteSPM bridge and mirror its safety contract (P-17). Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note), NOT STARTED - added under this package's IN_REVIEW status. Folds proposed task P-17 from docs/REFERENCE-SYNTHESIS-AND-V1-FEATURE-ALIGNMENT-2026-11.md section 8 (and Section 2) into this package.
- **013-T21** Package VERIFIED for V1. Status: DEFERRED. Evidence: TARGET. Detail: DEFERRED (owner A-62, 2026-11-10; see the package DEFERRED note) and BLOCKED (owner acceptance, T01).

### 014-conversation-history-management

Package status: IN_REVIEW. Source: `specs/014-conversation-history-management/TASKS.md`.

- **014-T01** Owner acceptance of SPEC and blockers. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - B-38 accepted as proposed (2026-11-10, sarv-projects) and treated as unblocking implementation by owner instruction; the whole-SPEC ACCEPTED/LOCKED flip, O-02 and O-03 remain pending, so SPEC and WIRING statuses are unchanged and this task is not DONE.
- **014-T02** Promote accepted interfaces into owner docs and schemas. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - `docs/schemas/sqlite-v31.sql` (side table `conversation_history_state`, deviation from the column-on-`conversations` wording, see docs/STORAGE.md) is wired and migration tests pass; STORAGE, EVENTS and BUILD-TASKS version text synced. NOT done: OpenAPI routes, `error-codes.schema.json` entries for ... (rest in TASKS.md)
- **014-T03** Storage: Conversation status, rename and archive with CAS and idempotency. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - storage layer done (2026-11-10): `ConversationHistoryStore` (`rename_conversation`, `set_conversation_archived`, `get_conversation_status`, `list_conversations_by_status`) in `crates/storage-sqlite/src/conversations.rs`; CAS, request-dedup replay and changed-payload conflict, rename event carries `title_digest` only, ... (rest in TASKS.md)
- **014-T04** Storage: search index, in-transaction maintenance and resumable backfill. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (unblocked by B-38 per owner instruction 2026-11-10; not reached in this slice; formerly BLOCKED on owner acceptance, T01). PARTIAL precedent: `resource_index.rs` tokenizer and HMAC term tokens.
- **014-T05** Storage: delete tombstone, sealed purge manifest and read fences. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (unblocked by B-38 per owner instruction 2026-11-10; not reached in this slice; formerly BLOCKED on owner acceptance, T01; B-38). PARTIAL precedent: `context_document_purge.rs` `begin_context_document_purge`.
- **014-T06** Conversation purge reconciler and startup resume. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (unblocked by B-38 per owner instruction 2026-11-10; not reached in this slice; formerly BLOCKED on owner acceptance, T01).
- **014-T07** Operator routes: rename, archive, unarchive and status-aware existing routes. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (unblocked by B-38 per owner instruction 2026-11-10; not reached in this slice; formerly BLOCKED on owner acceptance, T01).
- **014-T08** Operator route: Conversation search. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (unblocked by B-38 per owner instruction 2026-11-10; not reached in this slice; formerly BLOCKED on owner acceptance, T01).
- **014-T09** Operator routes: deletion impact, delete and deletion status. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (unblocked by B-38 per owner instruction 2026-11-10; not reached in this slice; formerly BLOCKED on owner acceptance, T01).
- **014-T10** Tauri bridge commands for history management. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (unblocked by B-38 per owner instruction 2026-11-10; not reached in this slice; formerly BLOCKED on owner acceptance, T01).
- **014-T11** Sidebar search UI with accessible states. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (unblocked by B-38 per owner instruction 2026-11-10; not reached in this slice; formerly BLOCKED on owner acceptance, T01).
- **014-T12** Row menu and header actions: rename, archive, unarchive, delete. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (unblocked by B-38 per owner instruction 2026-11-10; not reached in this slice; formerly BLOCKED on owner acceptance, T01).
- **014-T13** Cross-package delete fences (retrieval, context, memory, artifacts, logs). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (unblocked by B-38 per owner instruction 2026-11-10; not reached in this slice; formerly BLOCKED on owner acceptance, T01).
- **014-T14** Failure, race and canary matrix; create the canonical test target. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (unblocked by B-38 per owner instruction 2026-11-10; not reached in this slice; formerly BLOCKED on owner acceptance, T01).
- **014-T15** Performance budget and list virtualization. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (unblocked by B-38 per owner instruction 2026-11-10; not reached in this slice; formerly BLOCKED on owner acceptance, T01).
- **014-T16** Cross-layer trace for each user-visible operation (AGENTS.md section 4). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (unblocked by B-38 per owner instruction 2026-11-10; not reached in this slice; formerly BLOCKED on owner acceptance, T01).
- **014-T17** Qualification on the release rows. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (unblocked by B-38 per owner instruction 2026-11-10; not reached in this slice; formerly BLOCKED on owner acceptance, T01).
- **014-T18** Documentation sync. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (unblocked by B-38 per owner instruction 2026-11-10; not reached in this slice; formerly BLOCKED on owner acceptance, T01).
- **014-T19** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (unblocked by B-38 per owner instruction 2026-11-10; not reached in this slice; formerly BLOCKED on owner acceptance, T01).

### 015-task-control-delegation

Package status: IN_REVIEW. Source: `specs/015-task-control-delegation/TASKS.md`.

- **015-T01** Owner acceptance of SPEC and blockers. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED - B-39 accepted as proposed (2026-11-10); pending owner acceptance of this SPEC revision 0.1 (owner: repository owner, sarv-projects; Delegation and Product prepare the decision).
- **015-T02** Promote accepted interfaces into owner docs and machine schemas. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **015-T03** Typed TaskStatus enum and transition table (Rust). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - Task status is a `String` in `TaskRecord`/`TaskView`; transitions are checked ad hoc (for example the `"READY" | "RUNNING"` safe boundary in `lead_failover_service.rs`).
- **015-T04** Pause and resume: storage, domain and quiescence. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - no `request_pause`/`resume` in source; error codes exist.
- **015-T05** Typed steering with sibling spec revisions. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - `revise_saved_spec` in `crates/domain-task/src/lib.rs` revises a saved spec but has no steering class and is not admitted while the Task runs.
- **015-T06** Task-level cancel fan-out to child Attempts. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - Feature 005 plans a single-Attempt cancel; no fan-out exists.
- **015-T07** Handoff as an authorized new Attempt with late-result fencing. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - `POST /v1/tasks/{task_id}/lead-agent` and `TaskLeadHandoffService::execute` commit the lead change (`apps/litecoworkd/src/task_lead_handoff_operator.rs`, `crates/domain-task/src/lead_handoff.rs`, safe-boundary checks in `crates/storage-sqlite/src/lead_handoff_boundary.rs`); `prepare_lead_handoff_packet` exists. ... (rest in TASKS.md)
- **015-T08** Child-Attempt admission service with allowlist and B-39 Environment rule. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - decision logic and SQLite guard exist (`crates/domain-task/src/delegation_admission.rs`, `StepAttemptCoordinator::preview_delegated_admission` and `admit` in `crates/domain-task/src/execution.rs`, guard in `crates/storage-sqlite/src/execution.rs`); no production caller in `apps/litecoworkd`, no wire code for ... (rest in TASKS.md)
- **015-T09** Budgets and escalation with on_exhaustion (A-37). Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - `EscalationPolicy` and `EscalationAttemptOutcome` exist in `crates/domain-task/src/delegation_recovery.rs`; `delegation_budget_policy_json` is read by the SQLite guard; no `on_exhaustion`, no threshold action executor.
- **015-T10** Independent-agent verification. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - `start_verification_run`/`settle_verification_run` in `crates/storage-sqlite/src/verification_runs.rs` and `VerificationRunStatus` in `crates/domain-effects/src/lib.rs` exist; no producer and no independence rule.
- **015-T11** Operator routes, authorization and Tauri bridge commands. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL - `POST /v1/tasks/{task_id}/lead-agent` and `GET /v1/tasks/{task_id}/verification-runs` are served behind `authenticate`; no pause, resume, steering, activity or workers route; no Tauri bridge for any of them.
- **015-T12** Work-screen controls: Pause, Resume, Steer, Cancel, Change lead. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - `TaskDetailPanel` in `apps/litecowork-ui/src/App.tsx` has no execution controls.
- **015-T13** Activity timeline and Workers view. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - no timeline component; V1-UI-SCREEN-CONTRACT section 3 records the gap.
- **015-T14** Windows row: delegation and execution controls UNAVAILABLE until B-35. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **015-T15** Failure and recovery suite. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **015-T16** Cross-layer traces for every user-visible operation. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **015-T17** Linux-row qualification of the B-39 pair (CODE, SYSTEM, USER). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **015-T18** Windows-row qualification after B-35. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED - Feature 005 B-35 is PROPOSED (owner: repository owner) and no Windows Environment provider exists.
- **015-T19** Documentation and ledger sync. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **015-T20** Mission Control canvas and read-only Compare Agents fan-out (P-14). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - added under this package's IN_REVIEW status. Folds proposed task P-14 from docs/REFERENCE-SYNTHESIS-AND-V1-FEATURE-ALIGNMENT-2026-11.md section 8 into this package.
- **015-T21** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED

### 016-rich-presentation-intelligent-ui

Package status: IN_REVIEW. Source: `specs/016-rich-presentation-intelligent-ui/TASKS.md`.

- **016-T01** Owner acceptance of SPEC and blockers. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED - B-40 accepted as proposed (2026-11-10); pending owner acceptance of SPEC revision 0.1 (owner: repository owner, sarv-projects; Presentation Runtime, Product and Security prepare the decision).
- **016-T02** Promote accepted interfaces into owner docs and machine schemas. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **016-T03** Single Rust validator crate and drift fix. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - Rust validation exists inside `crates/storage-sqlite/src/rich_presentations.rs` (TEXT_SLICE and LAYOUT only, 1 MiB cap); no shared crate.
- **016-T04** Shared golden corpus and TypeScript parity. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - `rich-presentation.ts` and `tests/rich-presentation.test.ts` exist with inline fixtures; no shared corpus.
- **016-T05** Restricted CalcAst validator (Rust) and pure reducer (TypeScript). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **016-T06** CHART block with host SVG and table fallback. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - PRESENTATION-RUNTIME records CHART as a text placeholder today.
- **016-T07** SAFE_CALCULATOR Intelligent UI component (B-40). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **016-T08** PresentationBinder and publish after semantic commit. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - `publish_rich_presentation` has no production caller.
- **016-T09** Disable preference and semantic fallback (E2E-11). Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - `RichResponseView.tsx` already renders semantic text when no valid presentation exists and offers "Read the original response"; no preference to disable rich presentation.
- **016-T10** Generated media as provenance Artifacts. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - no ingestion pipeline; no Artifact create route today.
- **016-T11** Interactive Artifact Runtime kept PREVIEW_ONLY and inert. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - no runtime code exists and `PresentationRuntime.tsx` renders MCP_APP as unavailable; the Library/Workbench has no explicit inert preview for interactive Artifacts.
- **016-T12** No-executable-content gate, hostile corpus and CSP check. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - `presentation-runtime-fallback.test.mjs` covers escaping and unsupported kinds for `PresentationRuntime.tsx`; no static gate for the presentation directory; CSP is not tested.
- **016-T13** Failure and recovery suite. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **016-T14** Performance budgets. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **016-T15** Cross-layer traces for every user-visible operation. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **016-T16** Release-row qualification (E2E-11) on Windows x64 then Linux x64. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **016-T17** Documentation and ledger sync. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED
- **016-T18** Per-block Markdown degradation with code and TEXT_SLICE rendering (P-03). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - added under this package's IN_REVIEW status. Folds proposed task P-03 from docs/UI-RENDERING-SETTINGS-ARCHITECTURE-AUDIT-2026-11.md section 9 (findings R-05, R-06) into this package.
- **016-T19** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED

### 017-data-portability-backup-restore

Package status: IN_REVIEW. Source: `specs/017-data-portability-backup-restore/TASKS.md`.

- **017-T01** Owner acceptance of SPEC and blockers. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED - B-41 (backup scope, encryption at rest, restore-over-live policy) accepted as proposed (2026-11-10, sarv-projects); whole-SPEC acceptance and the ACCEPTED/LOCKED flip remain pending. The package is IN_REVIEW.
- **017-T02** Promote accepted interfaces into owner docs and schemas. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - OpenAPI has four target-only backup operations; no backup error codes, operation table or import route exist.
- **017-T03** Workspace-scoped snapshot, table map and secret canary scan. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - `build_filtered_snapshot` carries every Workspace's rows; exclusion is a deny list only.
- **017-T04** Restore correctness fixes, staging sweep and Windows private directories. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - `restore_into_staging` sets `restored_at: manifest.verified_at`; no new Runtime identity is minted; no startup sweep; Windows `ensure_private_directory` applies no DACL.
- **017-T05** Portable export and import re-encryption. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - blob envelopes are copied verbatim with `BackupKeyReference` only; restore needs the source keystore keys.
- **017-T06** Durable backup operation record, single-flight, progress and cancellation. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - `create_backup` is one blocking call with no progress or cancellation; no operation table.
- **017-T07** Daemon BackupService and Operator routes. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - no backup route is mounted; the four OpenAPI operations are target-only.
- **017-T08** Live import of an absent Workspace. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - only `restore_backup` into an empty destination exists.
- **017-T09** Offline whole-installation restore from the desktop. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - restore runs only from the CLI; there is no authenticated Operator stop (`main.rs` HELP says so).
- **017-T10** Tauri bridge commands. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - no backup command in `apps/litecowork-ui/src-tauri/src`.
- **017-T11** Settings "Storage & Backup" UI. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - `SettingsCategory` has no storage-backup entry.
- **017-T12** Deletion and purge status with retry. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL - per-Resource ContextDocument deletion read and request routes are served (`operator.rs`, `context_document_deletion_bridge.rs`); no Workspace-level list or Settings surface exists.
- **017-T13** Failure and recovery suite and canonical test target. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - storage tests cover corrupt archive, wrong passphrase, failed restore, non-empty destination, live lock and failed create; no crash-window, cancellation race or route-level replay test; `apps/litecoworkd/tests/backup_restore.rs` does not exist.
- **017-T14** Performance baseline and streaming bounds. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - container uses 1 MiB chunks; no measured memory or latency baseline.
- **017-T15** Cross-layer trace for each user-visible operation. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **017-T16** Qualify E2E-12 on the release matrix rows. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **017-T17** Documentation and inventory sync. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **017-T18** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.

### 018-usage-diagnostics-observability

Package status: IN_REVIEW. Source: `specs/018-usage-diagnostics-observability/TASKS.md`.

- **018-T01** Owner acceptance of SPEC and blockers. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED - B-42 (support-bundle contents and redaction allowlist) is PROPOSED, pending the repository owner (sarv-projects). The package is IN_REVIEW.
- **018-T02** Promote accepted interfaces and the vocabulary mapping. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - usage routes are target-only; no diagnostics routes or error codes exist; `attribution_source` (Feature 001) and `UsageConfidence` (LLD) are not mapped in any owner doc.
- **018-T03** Usage normalization contract for Feature 001 settlement. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - no `usage.rs` in `crates/storage-core/src`.
- **018-T04** Extract typed usage counters from the OpenCode v2.0.32 adapter. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - `opencode_server.rs` parses no token, cost or usage field; O-01 is open.
- **018-T05** Usage ledger reader and projections. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - no reader exists; depends on Feature 001's usage table.
- **018-T06** Usage Operator routes and usage export. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - `getConversationUsage` is target-only; no Workspace usage route.
- **018-T07** Structured redacted operational log replacing eprintln. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - 12 `eprintln!` sites; no log sink.
- **018-T08** Local crash records for daemon and desktop. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - no panic hook in `apps/litecoworkd` or `apps/litecowork-ui/src-tauri`.
- **018-T09** Health probe projection and build info. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - `GET /v1/runtime-lifecycle` and `litecoworkd status` exist; no per-probe projection with age and no build-info route.
- **018-T10** Support bundle preview and write (B-42). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **018-T11** Tauri usage and diagnostics bridges. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - only `agent_quota_bridge.rs` exists.
- **018-T12** Settings "Usage & Monitoring" and "About & Diagnostics" UI. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - neither category exists in `SettingsView.tsx`; `QuotaObservation.tsx` is the only usage-related component.
- **018-T13** Failure, recovery and backpressure suite with the canonical test target. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - `apps/litecoworkd/tests/usage_diagnostics.rs` does not exist.
- **018-T14** Usage query performance baseline. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - no baseline; LLD forbids hard-coded SLOs before profiling.
- **018-T15** Cross-layer trace for each user-visible operation. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **018-T16** Qualify on the release matrix rows. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **018-T17** Documentation and inventory sync. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **018-T18** Generate operator-api.ts and TS validators from the Rust wire types with a drift gate (P-18). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED - added under this package's IN_REVIEW status. Folds proposed task P-18 from docs/REFERENCE-SYNTHESIS-AND-V1-FEATURE-ALIGNMENT-2026-11.md section 8 (and audit finding X-04) into this package.
- **018-T19** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.

### 019-release-engineering-qualification

Package status: IN_REVIEW. Source: `specs/019-release-engineering-qualification/TASKS.md`.

- **019-T01** Owner acceptance of SPEC and blockers B-43, B-44. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (B-43 and B-44 DEFERRED 2026-11-10 as release dependent; Windows rows deferred for later laptop testing; pending owner acceptance of SPEC revision 0.1, B-43 signing identity and update channel, B-44 licensing/packaging decision; owner: repository owner, sarv-projects).
- **019-T02** Promote release contracts into owner docs and schemas. Status: DONE. Evidence: CODE. Detail: DONE (CODE/DOCS) pending T01 owner acceptance: `docs/schemas/release-evidence-bundle.schema.json`, `docs/schemas/release-matrix-row.schema.json`, `tests/test_release_schemas.py` (9 tests), `docs/release/README.md`, ADR 0026 skeleton (PROPOSED, not accepted), ACCEPTANCE section 7 declared rows and NOT_COUNTED rule, TESTING ... (rest in TASKS.md)
- **019-T03** CI skeleton on a native Linux runner (Wave 0). Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (CODE, not run on GitHub): `.github/workflows/architecture-docs.yml` (jobs check, ui, tauri; the filename and shape are pinned by existing tests in `tests/test_architecture_validator.py`, so it is used instead of the `ci.yml` named below), `.github/CODEOWNERS` (repository owner only until teams exist) and ... (rest in TASKS.md)
- **019-T04** Native Windows x64 CI runner. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **019-T05** Secret scan and dependency audit as release gates. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (CODE): the secret scan now masks the matched bytes in its preview (it previously echoed the whole line including the token), supports a reviewed exceptions table (`scripts/secret-scan-exceptions.json`: id, path, pattern, reason, reviewer, expiry; expired or incomplete entries fail), runs on tracked source in the ... (rest in TASKS.md)
- **019-T06** CSP and Tauri capability hardening audit. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (CODE): CSP now carries `object-src`, `base-uri`, `frame-ancestors` and `form-action` set to `'none'`; `scripts/v1_csp_audit.py` and `tests/test_csp_audit.py` (8 tests, fixtures for unsafe-eval, remote script origin, shell/fs permission, removed object-src) pass, and `v1_packaging_check.py` calls the audit. `style-src ... (rest in TASKS.md)
- **019-T07** IPC peer-identity negative tests on the packaged build. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (source IPC exists; packaged negative tests do not).
- **019-T08** Windows x86_64-pc-windows-msvc installer (NSIS per-user, MSI). Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - `tauri.conf.json` bundle block, icons (`icon.ico`, `icon.png`), `tauri.bundle.conf.json` externalBin and `build-daemon-sidecar.mjs` exist; targets are `"all"`; no Windows installer options; `v1_packaging_check.py` cannot find a Windows daemon binary.
- **019-T09** Windows code signing (B-43). Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (B-43 pending owner; owner: repository owner with Security).
- **019-T10** Linux x64 deb package with signed checksums. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (CODE): `bundle.linux.deb` in `tauri.conf.json` now declares `libwebkit2gtk-4.1-0` and `libgtk-3-0` dependencies and installs `litecoworkd.service` to `/usr/lib/systemd/system` with no maintainer script (so it ships disabled); `scripts/linux-release-checksums.sh` signs and verifies `SHA256SUMS` with a detached GPG ... (rest in TASKS.md)
- **019-T11** Updater gated on Feature 011 quiescence (B-43). Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (B-43 pending owner; Feature 011 authenticated stop and update quiescence gate not in source, owner Runtime).
- **019-T12** Update panel UI states hosted in the Feature 011 Advanced tab. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (CODE): `src/updates/update-state.ts` (pure reducer, state descriptions, single-flight controller), `desktop-update-api.ts` (typed parsing of `check_for_update` / `install_update` results), `UpdatePanel.tsx` (aria-live status, focus moves to the result message, disabled install exposes its reason, no animation), ... (rest in TASKS.md)
- **019-T13** Licensing/packaging decision record and third-party notices (B-44). Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (B-44 pending owner; owner: repository owner).
- **019-T14** Row-aware release qualification matrix. Status: DONE. Evidence: CODE. Detail: DONE (CODE): `scripts/v1_release_matrix_rows.json` (two declared rows, macOS UNSUPPORTED, Claude Code rows NOT_RUN, Windows E2E-04, E2E-08 and terminal_pty UNAVAILABLE per B-17/B-35), `v1_platform_matrix.py` (WSL2, container and CI_IMAGE detection, NOT_COUNTED rule, schema-validated bundles, PASSED only with a linked bundle on ... (rest in TASKS.md)
- **019-T15** Evidence bundle process and owner USER sign-off record. Status: DONE. Evidence: CODE. Detail: DONE (CODE): `scripts/v1_evidence_bundle.py` (`assemble` and `validate`; schema validation, secret scan of the bundle and every attachment with redacted findings, attachment digests re-checked so editing one invalidates the bundle, path traversal refused, redacted index line only), `tests/test_evidence_bundle.py` (9 tests), ... (rest in TASKS.md)
- **019-T16** App-wide accessibility qualification. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - focused a11y tests exist (`apps/litecowork-ui/tests/settings-accessibility.test.mjs`, `tests/artifact-editor-focus.test.ts`, `tests/coworker-focus-plan.test.ts`, motion guards) and `scripts/check-rendered-ui.mjs` drives the rendered UI; no app-wide audit; the browser a11y run timed out (gap analysis).
- **019-T17** Performance and soak qualification. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - `scripts/v1_soak_harness.py` exists (Python `sqlite3`, not the daemon); BENCHMARKS budgets exist; no measurement of the packaged app or daemon.
- **019-T18** Resilience matrix on both rows against shipped storage code. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL (CODE): `crates/storage-sqlite/tests/resilience.rs` (3 native tests over the shipped store: committed write survives restart with `integrity_check` ok, uncommitted foreign transaction rolled back at restart, page-capped write refused with a storage-full error and the store stays consistent; no `cfg` skip) pass on Linux; ... (rest in TASKS.md)
- **019-T19** Desktop stop/update qualification gate journey on the packaged app. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (ACCEPTANCE marks the gate future and blocked).
- **019-T20** Cross-layer trace for release-owned user operations. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **019-T21** Documentation and ledger sync. Status: PARTIAL. Evidence: CODE + SERVED. Detail: PARTIAL (docs): `CURRENT-RUN.md` and `docs/IMPLEMENTATION-STATUS.md` now record the Wave 0 release-machinery and shell progress honestly, `docs/LITECOWORK-BUILD-TASKS.md` maps BT-00.7 and BT-10.5, the served-route inventory was regenerated (175/1/55) and `audit_served_routes.py` matches it. NOT done: Area 10 row-by-row evidence ... (rest in TASKS.md)
- **019-T22** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **019-T23** V1 release go/no-go. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.

### 020-proactivity-quiet-learning

Package status: IN_REVIEW. Source: `specs/020-proactivity-quiet-learning/TASKS.md`.

- **020-T01** Owner acceptance of SPEC, D-01 and blocker B-45. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (D-01 and B-45 accepted as proposed 2026-11-10; pending owner acceptance of SPEC revision 0.1; owner: repository owner, sarv-projects).
- **020-T02** Promote accepted interfaces into owner docs and schemas. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **020-T03** Fix the deterministic learning gate defects (Rust). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (defects present in `crates/storage-core/src/coworker_learning.rs`).
- **020-T04** SuggestionService.propose and the admission transaction. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (no `propose`; schema indexes for dedupe and cooldown exist).
- **020-T05** Deterministic producers and restart-safe cursor (D-01). Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **020-T06** Quiet hours on Suggestion and memory push surfaces. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED (QuietHours domain type exists in Feature 010's domain; no Suggestion or learning event class).
- **020-T07** Ideas page and Home idea completion. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - `SuggestionsPage.tsx` has visibility tabs, accept/dismiss/snooze/unsnooze, preference mute with unresolved-mutation recovery and a live region; Home and drawer limits and the provenance card content are not verified.
- **020-T08** Learning storage with CAS, idempotency and Resource-backed payloads. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (B-45 pending owner; owner: repository owner).
- **020-T09** LearningService and LearningClassifier port (B-45). Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (B-45 pending owner; Feature 009 Coworker-owned Conversation admission absent, owner Coworker domain).
- **020-T10** Learning Operator routes, authorization and Tauri bridge. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (B-45 pending owner).
- **020-T11** Learning review list, Remembered notice, policy and memory controls. Status: BLOCKED. Evidence: TARGET. Detail: BLOCKED (B-45 pending owner); `CoworkerSettings.tsx` shows the inactive-learning disclosure today.
- **020-T12** Goals and Needs You owner conformance. Status: PARTIAL. Evidence: CODE. Detail: PARTIAL - routes, services, UI and Needs You tests exist; no negative test that Goal commands create no Task or schedule; no explicit no-mutation assertion for Needs You.
- **020-T13** Failure, race and recovery suite. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **020-T14** Cross-layer trace for each user-visible operation. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **020-T15** Qualification on the release matrix rows. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **020-T16** Documentation and ledger sync. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.
- **020-T17** Package VERIFIED for V1. Status: NOT STARTED. Evidence: TARGET. Detail: NOT STARTED.

## 6. Next tasks in wave order

Wave definitions are in `docs/LITECOWORK-BUILD-TASKS.md` ("V1 release waves"). A wave starts when its inputs are met, not on a date. Do the owner housekeeping in section 8 first because it is cheap and unblocks CI claims. Paths are relative to the repository root. Read the whole task (contract, change intent, acceptance, cross-layer trace, DEFERRED/BLOCKED note) before editing.

### 6.0 Housekeeping first (no wave)

1. Convert to LF where the working tree is still CRLF: `scripts/*.sh`, `.uv-version`, `.python-version`, `rust-toolchain.toml` (owner confirmation requested, section 8; Feature 019 T03 pins them LF in `.gitattributes`). Verify with `file`.
2. Decide the fate of `stash@{0}` (section 3).
3. Re-run the seven gates and the baselines in section 2.

### 6.1 Wave 1 remainder (Features 001, 002, 003, 014, then 012)

1. **014 routes, search, purge** (unblocked by B-38): `specs/014-conversation-history-management/TASKS.md` T04 to T19. The storage layer `ConversationHistoryStore` (crates/storage-sqlite, migration `docs/schemas/sqlite-v31.sql`) exists. Missing: Operator routes (OpenAPI in `docs/schemas/`, handlers next to `apps/litecoworkd/src/conversation_operator.rs`), search index and purge manifest (need a new migration number after the allocation table in `docs/LITECOWORK-BUILD-TASKS.md`), UI (`apps/litecowork-ui/src/conversations/`).
2. **012 service, routes, UI** (unblocked: 001-T07 and T11 DONE): `specs/012-conversation-coedit-take-control/TASKS.md` T02 to T04 then T06 to T11. The storage layer (`docs/schemas/sqlite-v32.sql`, `conversation_control_events`, `conversation_owner_side_notes`) exists.
3. **002 remaining**: `specs/002-native-agent-install-auth-binding/TASKS.md`: T08 replay-safe credential slot submission (BLOCKED: needs a new migration number for the credential receipt), T12 Claude Code CLI AgentModule (BLOCKED: no Claude Code CLI installed to pin; a pin needs a binary or a captured fixture), T19 package VERIFIED. Missing Tauri commands for Sign in, Open native config and the model picker: add in `src-tauri` (cannot be compiled here; compile on the laptop). Migrations v33 (002-T04 descriptor store) and v34 (002-T07 auth observations) are allocated but unwritten; T04 currently reuses the existing migration-16 table with one recorded deviation.
4. **001 remainder**: `specs/001-conversation-session-supervisor/TASKS.md` T09 (SYSTEM and USER evidence, needs a live OpenCode v2.0.32), T12 (BLOCKED, no evidenced input-request event), T28 (VERIFIED package, depends on T09), and the PARTIAL items T01, T03, T04, T05, T14, T21, T22, T23, T25, T27 (gaps are written in each status line).
5. **003 Linux work**: `specs/003-workspace-roots-context-ingestion/TASKS.md` T01 (not started), T07, T09 to T18 (not started; migration v35 for `context_attachments` and `retrieval_receipts` is allocated, unwritten). T03, T05, T06 and the Windows part of T04 are DEFERRED to the laptop.

### 6.2 Wave 2 (needs 001 commit and recovery tasks, which are done at CODE level)

- `specs/005-safe-task-path/TASKS.md` (Linux only; v36). T16 to T18 (Windows Environment provider B-35) are BLOCKED.
- `specs/007-professional-artifacts/TASKS.md` (v37; T12 BLOCKED).
- `specs/009-coworkers-scoped-memory/TASKS.md` (v38; T16 BLOCKED).
- `specs/016-rich-presentation-intelligent-ui/TASKS.md` (no migration; per-block Markdown T18).
- `specs/013-litespm-package-lifecycle-bridge/TASKS.md`: every task DEFERRED by the owner (section 9); do not start.

### 6.3 Wave 3

- `specs/006-golden-connection-capability-write/TASKS.md` (v40; T02 and T05 BLOCKED on the B-19 follow-up).
- `specs/008-workbench-browser-terminal-computer/TASKS.md` (v41; T09 and T12 BLOCKED; iframe sandbox guard T19, Git worktree/diff/PR T20).

### 6.4 Wave 4

- `specs/004-agent-relay/TASKS.md` (no migration), needs 002-T12.
- `specs/015-task-control-delegation/TASKS.md` (v42; Mission Control and read-only Compare T20).
- `specs/020-proactivity-quiet-learning/TASKS.md` (v43).

### 6.5 Wave 5 and 6

- `specs/010-automations-recurring-host/TASKS.md` (v44; T08, T09, T15, T18 BLOCKED; T19 deferred out of V1).
- `specs/017-data-portability-backup-restore/TASKS.md` (v45), `specs/018-usage-diagnostics-observability/TASKS.md` (v46 only if needed; T01 and T10 wait for B-42), `specs/019-release-engineering-qualification/TASKS.md` (T06 to T23; T01, T09, T11, T13 release dependent, B-43 and B-44).

### 6.6 In parallel on the laptop

The Windows rows, the `src-tauri` build, the first live OpenCode run (closes 001-T09 and the O-05 input budget), see section 9.

### 6.7 Migration allocation (source: docs/LITECOWORK-BUILD-TASKS.md "Migration allocation")

| Migration | Owner | Status |
|---|---|---|
| v28 | Workspace delegation settings revisions (A-40), wired by 001 | wired |
| v29 | 001 dispatch outbox, single-flight triggers, commit identity, usage records | wired |
| v30 | 001-T25 REPORTED Conversation turn activity records | wired |
| v31 | 014-T03 `conversation_history_state` side table | wired (storage only) |
| v32 | 012-T05 `conversation_control_events`, `conversation_owner_side_notes` | wired (storage only) |
| v33, v34 | 002-T04 descriptor store, 002-T07 auth observations | allocated, unwritten (renumbered from v30, v31) |
| v35 | 003 `context_attachments`, `retrieval_receipts` | allocated (renumbered from v32) |
| v36 | 005 | allocated |
| v37 | 007 | allocated |
| v38 | 009 | allocated |
| v39 | 013 | allocated, DEFERRED |
| v40 | 006 | allocated |
| v41 | 008 | allocated |
| v42 | 015 | allocated |
| v43 | 020 | allocated |
| v44 | 010 | allocated |
| v45 | 017 | allocated |
| v46 | 018 | allocated only if needed |

Packages 004, 011, 016 and 019 plan no migration. The 014 search index and purge tables and the 002-T08 credential receipt need numbers after v46. Rules: the first package to bump `SCHEMA_VERSION` also wires earlier ones (migrations apply in order); a package merging out of order takes the next free number and updates the table in the same change; a merged migration is never renumbered or edited.

## 7. Pitfalls

1. **v28 fixture snapshot column.** v28 added `delegation_policy_snapshot_json` to `task_spec_revisions`, and the two TaskSpec revision writers in `crates/storage-sqlite/src/lib.rs` now resolve and pin it from the Workspace settings head. Every full-schema test fixture that inserts a TaskSpec revision must supply a snapshot. A fixture without one fails with a NOT NULL or CHECK error that looks unrelated to the test. The labelled `stash@{0}` was an earlier attempt at the same fix for `environment_tests.rs` and `lead_failover_choice_request.rs`.
2. **Auto-created `workspace_delegation_settings`.** Because the writers resolve the settings head, the first TaskSpec revision for a Workspace auto-creates a default `workspace_delegation_settings` row. Tests that count rows in that table, or assert it is empty after a Task write, see one extra row. Assert on the Workspace you created, not on table totals.
3. **Always use `--lib` for the storage-sqlite filter.** `cargo test -p litecowork-storage-sqlite <filter>` without `--lib` also builds and runs the integration target `tests/agent_registry_cache.rs` and doc-tests, which is slow and can mask a zero-match filter. Use `cargo test -p litecowork-storage-sqlite --lib <filter>`; check the "N passed" line is non-zero so a typo does not read as success.
4. **Run the full suite before any migration commit.** A new migration or a `SCHEMA_VERSION` bump changes version assertions, table lists, fixture snapshots, generated SQL checks and the schema contract tests in several crates, not only the one you edited. Run `cargo test -p litecowork-storage-sqlite --lib`, `cargo test -p litecoworkd --bin litecoworkd`, then `python3 scripts/validate_architecture.py` before committing a migration.
5. **Stale-DEGRADED runtime test lesson.** Commit 00c4542 made the Runtime record DEGRADED with blocker `OPERATOR_ENDPOINT_FAILED` when the Operator endpoint dies after READY. Existing runtime process tests expected a stale READY after the endpoint died and failed once this was true. The lesson: when you make a status truthful, fix the tests that encoded the stale value instead of weakening the new behavior, and re-run the whole runtime test family.
6. **Migration renumbering.** Current mapping: v30 is 001, v31 is 014, v32 is 012, v33 and v34 are 002, v35 is 003, then 005=v36, 007=v37, 009=v38, 013=v39, 006=v40, 008=v41, 015=v42, 020=v43, 010=v44, 017=v45, 018=v46. Many packages once wrote "sqlite-v29.sql or the next free number" and 012 once named v30; collisions were resolved on 2026-11-10 across 25+ spec files (DATA-MODEL, PLAN, README, RESEARCH, TASKS). If you add or move a migration, update `docs/LITECOWORK-BUILD-TASKS.md` and every package that names the number in the same change. A merged migration is never edited or renumbered.
7. **Validator duplicate full task ID failure.** `.specify/scripts/validate_feature_package.py` fails when a full `NNN-TNN` ID appears twice in one package TASKS.md. In notes and Reference lines write "task T12 of package 011". PROPOSED DeepSeek rows are labelled `PROP-Pnn` for the same reason.
8. **Obsolete `stash@{0}` check.** See section 3: diff before dropping; never pop.
9. **Gate output churn.** Gates rewrite `docs/audit/*` with LF endings; run `git checkout -- docs/audit` last. `docs/COMPLETE-ACTIVE-CORPUS-INVENTORY.md` is regenerated and kept.
10. **CRLF.** New Markdown must be CRLF. Create with LF then convert (`sed -i 's/$/\r/'` on a file known to be LF only), and check with `file`. The shell scripts, version pins and `rust-toolchain.toml` must be LF.
11. **Tauri code is uncompiled here.** Any change in `src-tauri` must be flagged as uncompiled in the commit message and the TASKS.md status line.
12. **Cargo and sandbox temp.** `/tmp` does not persist between tool calls; scratch files go under `/projects/sandbox/.work`.
13. **Status honesty.** A task is DONE only with its stated acceptance evidence. Where no live agent exists, the status line says "without a live native adapter" or similar; preserve that wording.

## 8. Open owner decisions

Owner: sarv-projects. Accepted on 2026-11-10 (register A-62): B-38 (delete semantics), D-01 (first Suggestion producers), B-45 (first learning source), B-39 (first heterogeneous delegation pair), B-40 (first Intelligent UI component), B-41 (backup scope), the B-31 destination class (follow-up class), CONNECTOR_EVENT over LOCAL_POLL, E2E-14 to E2E-17, and 010-T19 stays deferred out of V1. Packages 013 to 020 stay IN_REVIEW until the whole SPEC of each is accepted. The A-62 accepted list above is complete.

Open:

- **B-42** (018-T01 and 018-T10): support-bundle allowlist. Blocks those two tasks only.
- **B-19 follow-up** (006-T02 and 006-T05): the GitHub OAuth client-secret problem for a local desktop app. Blocks the golden Connection capability write.
- **P-38**: login-shell PATH-class capture for agents launched from a Linux desktop entry versus the A-58 environment scrubbing rule. Class OWNER, no reuse allowed before the decision (section 10).
- **Housekeeping CRLF to LF**: convert `scripts/*.sh`, `.uv-version`, `.python-version` and `rust-toolchain.toml` where still CRLF in the working tree.
- **Secret-scan fixture exceptions**: owner confirmation of the entries in `scripts/secret-scan-exceptions.json` (used by `scripts/v1_secret_scan.py`).
- Smaller open items recorded in package files: O-05 (real OpenCode input budget, 001-T05) and 002-T05, both resolved only by a live run.

## 9. Deferred work and where to test it later

Owner instruction of 2026-11-10 (register A-62): defer and note, do not fake.

### 9.1 All of 013 LiteSPM

Every task in `specs/013-litespm-package-lifecycle-bridge/TASKS.md` is DEFERRED, with a package note at the top. Why: owner deferral; LiteSPM service, version, protocol and receipt handling must be qualified before package lifecycle mutations. The pin is LiteSPM 84d5fab. Unblocks when the owner lifts the deferral and accepts the 013 SPEC. Test later at: the LiteSPM integration plan in `docs/LITESPM-INTEGRATION-PLAN.md`, `docs/LITESPM-LITECOWORK-INTEROP-LLD.md` and `docs/LITESPM-INTEGRATION-STUDY-2026-11.md`; migration v39 is reserved.

### 9.2 Windows rows

Examples: 003-T03, T05, T06 and the Windows part of T04 (Windows root add, scan, reparse-point and path-identity matrix); the Windows part of 001-T14; Windows Task execution, daemon terminal and delegation (005-T16 to T18, B-35); the Windows IPC profile (011-T02, T03). Test later on the owner's native Windows x64 laptop first (release matrix row 1: Windows x64 with OpenCode v2.0.32), then native Linux x64. WSL2, containers and CI runners never count. Also compile `src-tauri` there.

### 9.3 Release-dependent rows

B-43 signing and update channel, B-44 licensing and packaging, packaging, release-matrix qualification, the V1 go/no-go (019-T01, T09, T11, T13, T23). Test later at: `docs/release/`, `scripts/qualify-linux-package.sh`, `scripts/qualify-linux-systemd-runtime.sh`, `scripts/v1_release_matrix_rows.json`, `scripts/v1_evidence_bundle.py`, the USER sign-off template, and the first GitHub Actions run.

### 9.4 Where the per-task reasons live

Each deferred or blocked task has a `DEFERRED/BLOCKED note` line (why, unblocks when, test later at) in its TASKS.md: 41 notes plus the 013 package note. Other BLOCKED items: 001-T12, 002-T08, 002-T12, 005-T16 to T18, 006-T02, T05, 007-T12, 008-T09, T12, 009-T16, 010-T08, T09, T15, T18. DEFERRED outside 013 and Windows: 010-T19 (out of V1), 011-T02, T03, T17, 002-T15.

## 10. Study documents and the DeepSeek Harness reuse map

### 10.1 DeepSeek Harness (MIT, copyright 2026 DeepSeek, upstream https://github.com/deepseek-ai/deepseek-harness, pinned commit fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159)

- [docs/DEEPSEEK-HARNESS-REUSE-MAP-2026-11.md](DEEPSEEK-HARNESS-REUSE-MAP-2026-11.md): maps each proposal P-27 and P-31 to P-46 (and plugin patterns) to the upstream file with a permalink at the pinned commit, the owning LiteCowork task and the class: ADOPT (same algorithm and invariants re-implemented in LiteCowork's language, called PORT in earlier revisions; the upstream file is still not imported), ADAPT (reimplement the pattern with LiteCowork-specific inputs), REJECT (rationale given) or OWNER (needs an owner decision). It also lists the rejected table. **No upstream code has been copied.** Upstream is Electron and TypeScript, LiteCowork is Tauri, Rust and React, so nothing is used as-is: each task states whether it is an ADOPT (same algorithm, re-implemented), an ADAPT or a REJECT. Any later copy must carry the MIT notice and update the map (AGENTS.md section 7).
- [docs/DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md): desktop shell deep-dive (register A-63, proposals P-31 to P-46, P-38 conflict note in Section 14). Second pass with line-level scripts, src and installer findings (commit 340912b).
- [docs/DEEPSEEK-HARNESS-PLUGIN-ARCHITECTURE-DEEP-DIVE-2026-11.md](DEEPSEEK-HARNESS-PLUGIN-ARCHITECTURE-DEEP-DIVE-2026-11.md): plugin and capability architecture deep-dive (register A-58).
- In packages 002, 005, 008, 010, 011, 013, 017, 018 and 019 (plus RESEARCH in 001 and 006) there are PROPOSED `PROP-Pnn` rows and PLAN and RESEARCH sections. Each affected task has a `Reference (DeepSeek Harness ...)` line with full permalinks (506 permalinks, all verified to exist in the `repo_take` clone). They are not accepted tasks. Owner review is required before any becomes a numbered task. On acceptance: assign the next free task ID, copy the permalink, class and reuse statement into the task and update the reuse map.
- P-38 stays OPEN (conflict with A-58).
- The generator that wrote the rows is `/projects/sandbox/.work/ds_sync.py` (idempotent; sandbox artifact, not in the repository).

### 10.2 Other study and audit documents

- [docs/RIVAL-DEEP-DIVE-PLUGIN-AND-CAPABILITY-PATTERNS-2026-11.md](RIVAL-DEEP-DIVE-PLUGIN-AND-CAPABILITY-PATTERNS-2026-11.md): eight rival plugin and capability architectures.
- [docs/OPEN-DESIGN-CAPABILITY-DEEP-DIVE-2026-11.md](OPEN-DESIGN-CAPABILITY-DEEP-DIVE-2026-11.md): line-level deep-dive of open-design.
- [docs/ZCODE-PARADEV-CODING-COCKPIT-DEEP-DIVE-2026-11.md](ZCODE-PARADEV-CODING-COCKPIT-DEEP-DIVE-2026-11.md): ZCode and paradev coding cockpit.
- [docs/REFERENCE-SYNTHESIS-AND-V1-FEATURE-ALIGNMENT-2026-11.md](REFERENCE-SYNTHESIS-AND-V1-FEATURE-ALIGNMENT-2026-11.md): reference synthesis and V1 feature alignment.
- [docs/LITESPM-INTEGRATION-STUDY-2026-11.md](LITESPM-INTEGRATION-STUDY-2026-11.md), [docs/LITESPM-INTEGRATION-PLAN.md](LITESPM-INTEGRATION-PLAN.md), [docs/LITESPM-LITECOWORK-INTEROP-LLD.md](LITESPM-LITECOWORK-INTEROP-LLD.md): LiteSPM.
- [docs/OPENBOT-REFERENCE-IMPLEMENTATION-STUDY.md](OPENBOT-REFERENCE-IMPLEMENTATION-STUDY.md): OpenBot, n8n and Activepieces workflow study.
- [docs/UI-RENDERING-SETTINGS-ARCHITECTURE-AUDIT-2026-11.md](UI-RENDERING-SETTINGS-ARCHITECTURE-AUDIT-2026-11.md), [docs/SETTINGS-AND-SHELL-SURFACE-AUDIT.md](SETTINGS-AND-SHELL-SURFACE-AUDIT.md): UI, rendering and Settings audits.
- [docs/COMPETITIVE-RESEARCH.md](COMPETITIVE-RESEARCH.md), [docs/SUPERSET-GAP-MATRIX.md](SUPERSET-GAP-MATRIX.md), [docs/REFERENCE-IMPLEMENTATIONS.md](REFERENCE-IMPLEMENTATIONS.md), [docs/REFERENCE-REPOS-CORE-ARCHITECTURE-INDEX.md](REFERENCE-REPOS-CORE-ARCHITECTURE-INDEX.md): competitor and reference indexes.
- [docs/CONTRADICTION-AND-ALIGNMENT-REGISTER.md](CONTRADICTION-AND-ALIGNMENT-REGISTER.md): the register (A-NN rows; A-52 to A-55 ACP and folded tasks, A-58 and A-63 DeepSeek, A-62 owner decisions).
- [docs/OWNER-ACCEPTANCE-REVIEW-PACKET.md](OWNER-ACCEPTANCE-REVIEW-PACKET.md): owner acceptance packet (Part 4 decisions).
- Status and trace: [docs/IMPLEMENTATION-STATUS.md](IMPLEMENTATION-STATUS.md), [docs/LITECOWORK-BUILD-TASKS.md](LITECOWORK-BUILD-TASKS.md), [docs/SPEC-TASK-TRACEABILITY.md](SPEC-TASK-TRACEABILITY.md), [docs/V1-FEATURE-SDD-REGISTER.md](V1-FEATURE-SDD-REGISTER.md), [docs/COVERAGE-MATRIX.md](COVERAGE-MATRIX.md), [docs/DOCS-REAUDIT-2026.md](DOCS-REAUDIT-2026.md), [specs/README.md](../specs/README.md), and the repository-level [CURRENT-RUN.md](../CURRENT-RUN.md) and [README.md](../README.md).
