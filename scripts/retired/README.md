# Retired validators

These scripts and their tests are preserved, not deleted. They are no longer wired into
scripts/check.sh and are intentionally outside the `tests/` discovery path.

| Script | Validated | Resolved by |
|---|---|---|
| validate_implementation_coverage.py | `implementation/machine-inventory.json`, `coverage.csv`, `ARCHITECTURE-COVERAGE.md` | The legacy planning workspace now lives only in docs/archive/legacy-implementation-c7e9c05/. Active coverage is docs/COVERAGE-MATRIX.md and docs/FEATURE-AND-UI-CONTRACT-COVERAGE.md. |
| validate_implementation_plan.py | `implementation/backlog.json`, `PROCESS.md`, `reviews/` | The epic backlog was retired in favour of specs/ feature packages and docs/LITECOWORK-BUILD-TASKS.md. |

Open capability deliberately not carried over: `api_documented_routes()` parsed the
`# Target only operation:` annotations in docs/API.md. `scripts/audit_served_routes.py` is the
active owner of the served/target-only split, so re-implement that parsing there if the
annotation is wanted in the machine inventory again.
