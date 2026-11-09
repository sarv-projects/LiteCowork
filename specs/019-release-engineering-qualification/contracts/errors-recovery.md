# Errors and Recovery Contract - Feature 019

**Status:** IN_REVIEW - not owner-accepted; authorizes no implementation.
**Owners:** Release Engineering, Security, Desktop Shell, QA, Runtime (Feature 011 gate).
**Canonical owners:** [FAILURE-RECOVERY](../../../docs/FAILURE-RECOVERY.md), [RUNTIME-LIFECYCLE](../../../docs/RUNTIME-LIFECYCLE.md), [ACCEPTANCE-AND-E2E-PLAN](../../../docs/ACCEPTANCE-AND-E2E-PLAN.md).

## 1. Build and gate failures

| Case | Action | Never |
|---|---|---|
| CI stage fails | Block merge; show stage and duration | Auto-retry until green |
| Host-dependent test red in a container or hosted image | Re-run on the real release-row OS (TESTING B-28 checklist) | Mark the row green from a sandbox |
| Run under WSL2, a container or CI image | Mark NOT_COUNTED | Count it for Windows or Linux desktop |
| Secret scan finding | Block; rotate with owner | Print the secret; ship |
| Dependency advisory without unexpired exception | Block | Silence by downgrading severity |
| Signing service unavailable | Block release | Ship unsigned |
| CSP/capability regression | Block | Relax without recorded reason |

## 2. Update failures and recovery

| Case | State | User sees | Recovery |
|---|---|---|---|
| Manifest unreachable or offline | CHECK_FAILED / OFFLINE | "Could not check for updates" with reason | Retry; never "Up to date" |
| Manifest or artifact signature invalid | FAILED (`SignatureInvalid`) | Update refused; current version unchanged | Report to Release Engineering |
| Offered version lower or equal | REFUSED (`Downgrade`) | No update offered | None needed |
| Gate returns NotQuiescent | REFUSED | List of running or unknown items from Feature 011 | Retry after work settles |
| Gate unavailable (Feature 011 not served) | REFUSED (`QuiescenceUnavailable`) | "Update needs the Runtime stop service, which is not available in this build" | Manual reinstall after explicit stop |
| Second install while one is active | `AlreadyInProgress` | Existing progress | Wait |
| Download interrupted | FAILED (`INTERRUPTED` on next launch) | Last attempt failed | Retry downloads again; partial file deleted |
| Runtime stop timeout | FAILED (`RuntimeStopFailed`) | What did not stop | Retry; no installer launched |
| Installer killed or failed | FAILED or COMPLETED on next launch by version check | Running version and attempt result | Old version keeps running; owner may retry |
| New version migration fails | Daemon DEGRADED with startup blocker | Startup blocker and restore option (Feature 017) | Restore backup; never auto-downgrade the database |
| Disk full during download | FAILED | Free space message | Retry after freeing space |

## 3. Release decision failures

A missing VERIFIED package, a claimed cell without USER sign-off, or a security gate not green on the release commit makes the decision NO-GO with the exact list. There is no override; the remedy is evidence or a narrower claim (an unsupported cell shown UNAVAILABLE).
