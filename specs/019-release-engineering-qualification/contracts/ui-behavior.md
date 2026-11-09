# UI behavior - Install, Update and About states (Feature 019)

**Status:** IN_REVIEW - not owner-accepted; authorizes no implementation. Binds UI to owner contracts; not pixels.
**Normative UI indexes:** [V1-UI-SCREEN-CONTRACT.md](../../../docs/V1-UI-SCREEN-CONTRACT.md), [V1-UI-ACTION-STATE-LEDGER.md](../../../docs/V1-UI-ACTION-STATE-LEDGER.md), [DESKTOP-SHELL-LLD.md](../../../docs/DESKTOP-SHELL-LLD.md).
**Host surface:** Feature 011 owns the Settings shell and the Advanced tab; this package supplies the `UpdatePanel` component it mounts and defines no Settings category. Feature 018 owns About and Diagnostics; this package supplies version, signature state and the last UpdateAttempt for display.

The UI renders typed results from Tauri commands; it does not parse English error strings and never computes quiescence.

## 1. User flows

| Step | Surface | Transition |
|---|---|---|
| F1 Install | Signed NSIS installer (per-user) | Install -> first launch -> Runtime readiness shown by Feature 011 |
| F2 Check for updates | Settings > Advanced > Updates (`UpdatePanel`) | IDLE -> CHECKING -> UP_TO_DATE / AVAILABLE / CHECK_FAILED / OFFLINE |
| F3 Install update | `UpdatePanel` Install button | AVAILABLE -> GATING -> REFUSED_NOT_QUIESCENT or INSTALLING -> HANDOFF |
| F4 Relaunch after update | App start | Last attempt shown as COMPLETED or FAILED with reason |
| F5 Uninstall | OS Apps settings | Binaries removed; data kept unless the owner opts in |
| F6 About | Feature 018 About page | Version, channel, signature state (`SIGNED` / `UNSIGNED-DEV`) |

## 2. UI states

| State | Meaning | Controls |
|---|---|---|
| IDLE | No check yet this session | Check for updates enabled |
| CHECKING | Manifest fetch in progress | Buttons disabled; `aria-busy` |
| UP_TO_DATE | Verified manifest shows no newer version | Check again |
| CHECK_FAILED | Fetch or verification failed | Reason; Retry |
| OFFLINE | No network | Reason; Retry |
| AVAILABLE | Newer signed version | Version, notes link, Install |
| GATING | Waiting for Feature 011 gate | Disabled; "Checking that nothing is running" |
| REFUSED_NOT_QUIESCENT | Gate returned NotQuiescent or unavailable | List of running or unknown items; Retry |
| INSTALLING | Download, verify, Runtime stop | Progress; no Cancel after stop begins |
| HANDOFF | Installer launched | "LiteCowork will restart" |
| FAILED | Typed failure | Reason; Retry |
| UNSIGNED_DEV | Running an unsigned build | Updates disabled with reason |
| LINUX_MANUAL | Linux row in V1 | "Update by installing the new package" |

## 3. Commands (UI -> core)

| UI control | Reducer action | Command |
|---|---|---|
| Check for updates | `CHECK_REQUESTED` | `check_for_update` |
| Install | `INSTALL_REQUESTED` with one UUID `request_id` | `install_update` |
| Retry | Re-issues the last command with a new request id | same |

## 4. Accessibility

All controls keyboard reachable in order; focus moves to the result message after each command; state changes announced through a polite live region; disabled controls expose the reason as accessible description; reduced motion removes progress animation; contrast meets WCAG 2.2 AA. App-wide qualification of every screen is TASKS T16.

## 5. Failure presentation

Every refusal shows a specific cause from `UpdateError` or the gate's reasons. "Up to date" is shown only after a verified manifest. Window close is never presented as Runtime stopped.

## 6. Cross-layer trace

| Operation | Control | Command | Owner route | Fence | Receipt | Recovery | Evidence |
|---|---|---|---|---|---|---|---|
| Install update | `UpdatePanel` Install | `install_update` | Feature 011 authenticated stop | single-flight mutex; quiescence verdict | UpdateAttempt record, installer exit code | settle on next launch | T11, T19 |
| Check for updates | `UpdatePanel` Check | `check_for_update` | none (HTTPS manifest) | signature, version order | none | Retry | T11, T12 |
| Install / uninstall | OS installer | n/a | n/a | per-user install path | installer log | reinstall | T08 |

Rows are completed with exact evidence by TASKS T20.
