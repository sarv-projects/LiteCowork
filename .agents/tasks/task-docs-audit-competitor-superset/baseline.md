# Audit Baseline — task-docs-audit-competitor-superset (FEAT-001)

Captured at the start of the assumption-free docs re-audit so that every later doc
diff is attributable to a deliberate change rather than a pre-existing drift.

- Repo: `litecowork`
- Base branch: `main`
- Base commit (HEAD at snapshot): `2e85f03`
- Feature branch created and checked out: `docs/audit-competitor-superset`
- Working tree before branch: clean (`nothing to commit, working tree clean`)
- Date of snapshot: start of FEAT-001.
- Host: Linux sandbox. Network OPEN.

## Python environment used for the gates

The doc gates import `yaml`, `jsonschema`, and `openapi_spec_validator`, which are
NOT present in the bare system interpreter (`python3` 3.11.15 via pyenv). The
canonical, project-pinned way to run them (per `scripts/check.sh`) is the locked uv
environment: `uv run --locked python <script>`.

- Pinned Python: `3.13.12` (`.python-version`)
- Pinned uv: `0.12.23` (`.uv-version`); host uv present: `0.13.5`
- `pyproject.toml` dev group: `jsonschema>=4.23,<5`, `openapi-spec-validator>=0.7,<1`, `PyYAML>=6,<7`
- `uv sync --locked` succeeded after redirecting uv's data/cache/python dirs to a
  writable location (the default `/root/.local/share/uv` is read-only in this sandbox):
  `UV_CACHE_DIR=/tmp/uv-cache UV_PYTHON_INSTALL_DIR=/tmp/uv-python XDG_DATA_HOME=/tmp/uv-data TMPDIR=/tmp`.
- Installed 20 packages incl. `jsonschema==4.26.0`, `openapi-spec-validator==0.9.0`, `pyyaml==6.0.3`.
- `.venv/` is gitignored (`.gitignore` line 26); it is NOT staged.
- No Rust/cargo build was needed for any doc gate; the TMPDIR/PKG_CONFIG cargo
  prefix was therefore not required here.

## Doc-validation gate results at HEAD 2e85f03 (verbatim)

All gates were run with `uv run --locked python <script>` (the pinned env).

| Gate | Result | Verbatim output / note |
| --- | --- | --- |
| `scripts/audit_v1_markdown.py` | PASS (exit 0) | `markdown audit: files=255 lines=54613 bytes=4288556 broken_links=0` |
| `scripts/audit_v1_design_depth.py` | PASS (exit 0) | `DESIGN_DEPTH files=258 lines=54701 domains=12 functions=254 settings_rows=187 routes=229 served=173 target=56 tables=107 packages=11 findings=0` |
| `scripts/validate_architecture.py` | PASS (exit 0) | `Architecture validation passed: JSON Schemas, 148 typed events, error codes, OpenAPI structure, SQLite, Gateway names, canonical concepts, product naming, and active Markdown links.` |
| `scripts/validate_design_owner_consistency.py` | PASS (exit 0) | `documents_read=259 lines_read=55003 contract_assertions=36 issues=[]` (sha256_document_index `8f19d385d8b0759408e1456c2e0c2ef80c1b9fe112af960328cc8a1344bcf86a`) |
| `scripts/audit_v1_contracts.py` | PASS (exit 0) | `V1_CONTRACT_AUDIT errors=0 openapi=229 served=173 outside=1 target=56` |
| `.specify/scripts/validate_feature_package.py` | PASS (exit 0) | `PASS: 11 feature package(s); required files, SPEC sections, one canonical bash command, all-package Markdown links, task fields/dependencies, acceptance traceability, WIRING edges and transaction boundaries validated` (note: structural validation only, not owner approval / runtime qualification) |
| `corpus_audit.py` (repo root) | RUNS but UNUSABLE | Exit 0, but reports `Total files read: 0 / Total lines read: 0` and empty JSON. Root cause: the script has a hardcoded Windows `base_dir = r'C:\Users\Sonali\Desktop\...\LiteCowork\docs'` (line 5) that does not exist on this Linux host, so `os.walk` iterates nothing. This is a script-portability bug, NOT a sandbox blocker and NOT a task failure. Ground-truth counts below are taken from the `scripts/` audit gates instead, as instructed. |

### Note on the markdown-audit scan root (affects all later re-runs)

`scripts/audit_v1_markdown.py` walks from the REPO ROOT and excludes
`.git`, `node_modules`, `target`, `.venv`, `.pytest_cache` and the generated
`docs/audit/*` reports, but it does NOT exclude `.agents/`. The baseline numbers
below (**255 files / 54,613 lines**) were captured with this `baseline.md` file
temporarily removed, so they reflect the true corpus at HEAD `2e85f03`.

With this 113-line `baseline.md` present, the same gate reports **256 files /
54,726 lines** (+1 file / +113 lines) because it counts the plan file itself.
Later phases should expect the audit count to include whatever `.agents/tasks/**`
markdown is on disk, and should reconcile against the +113-line offset of this
baseline file (and any further plan files) rather than mistaking it for corpus drift.

### Note on the `docs/audit/*` generated artifacts

`audit_v1_markdown.py` and `validate_design_owner_consistency.py` rewrite their
generated outputs (`docs/audit/V1-MARKDOWN-AUDIT.md`,
`docs/audit/V1-MARKDOWN-INVENTORY.csv`, `docs/audit/v1-cross-contract-guard.json`)
even without `--write`. On this Linux host those rewrites produced LF line endings
where the committed files use CRLF, so the only diff was line-ending noise (content
identical). To honor the FEAT-001 constraint that this feature touches NO file under
`docs/`, those rewrites were reverted with `git checkout -- <files>`, leaving
`git diff --name-only main -- docs specs` EMPTY. Later phases that legitimately
regenerate these artifacts must run the generator and review the output, never hand-edit.

## Ground-truth numbers RIGHT NOW (source: the `scripts/` gates)

- Active markdown corpus (markdown audit): **255 files / 54,613 lines** / 4,288,556 bytes / 0 broken relative links.
- Design-depth audit: **258 files / 54,701 lines**, 12 domains, 254 functions, 187 Settings rows, 229 routes (173 served / 56 target-only), 107 tables, 11 packages, **0 findings**.
- Design-owner consistency: 259 documents_read / 55,003 lines_read, 36 contract assertions, **0 issues**.
- Active-corpus manifest (`docs/COMPLETE-ACTIVE-CORPUS-INVENTORY.md`, excludes itself): per CURRENT-RUN.md it records 258 files; the design-depth generator now reports 54,701 lines for that same set (see drift below).

### OpenAPI operation counts (confirmed)

From `scripts/audit_v1_contracts.py` and `docs/schemas/served-route-inventory.json`
(`"version": 3`, method `composed_router_and_called_same_file_router_helpers`):

- **Served: 173** (`served` list length 173)
- **Served outside OpenAPI / authenticated-local-only: 1** (`served_outside_openapi` list length 1)
- **Target-only: 56** (`target_only` list length 56)
- **Total OpenAPI operations: 229** (173 served + 56 target-only). Plus 1 route served
  outside the OpenAPI document. Matches the briefing's "173 served / 1 authenticated-local-only / 56 target-only (229 total)".

### SDD package pass count

- **11 of 11** feature packages pass `validate_feature_package.py` (structural only).
  `specs/README.md` records 001 IN_REVIEW, 002-011 DRAFT.

## Line-count drift vs CURRENT-RUN.md (confirmed)

`CURRENT-RUN.md` currently prints stale numbers; the generators now report +14 lines on both:

| Metric | CURRENT-RUN.md cites | Generator reports now | Delta |
| --- | --- | --- | --- |
| Design-depth / active-corpus manifest lines | **54,687** (lines 86, 157, 159) | **54,701** | +14 |
| Markdown-audit lines | **54,599** (line 88) | **54,613** | +14 |
| Design-depth files | 258 | 258 | 0 |
| Markdown-audit files | 255 | 255 | 0 |

So the briefing's "generators report 54,701 / 54,613" is TRUE as of this snapshot;
CURRENT-RUN.md's 54,687 / 54,599 are STALE by 14 lines each. This is the known drift
(A-type danger) to reconcile in a later phase — NOT in FEAT-001.

## Next-unused global IDs reserved for Phases 1-3 (grep-confirmed)

ID allocation is global and append-only. Highest existing values found by grep over `docs/`:

| Kind | Highest existing | NEXT UNUSED (reserved) |
| --- | --- | --- |
| Contradiction / alignment `A-xx` | A-50 | **A-51** |
| Blocking decision `B-xx` | B-33 | **B-34** |
| Research-gate `RV-xx` (VIABILITY-VS-COMPETITORS.md) | RV-12 | **RV-13** |

## What FEAT-001 did and did NOT change

- DID: created the feature branch, synced the pinned uv Python env, ran every doc
  gate and captured verbatim output, confirmed ground-truth counts, confirmed the
  line-count drift, confirmed the next-unused A/B/RV IDs.
- DID NOT: modify any file under `docs/` or `specs/`
  (`git diff --name-only main -- docs specs` is empty). No competitor repos cloned
  yet (`repo_take/` not created). No product doc edited.
