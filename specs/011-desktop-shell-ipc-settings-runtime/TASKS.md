

# Tasks - Desktop Shell IPC Settings and Runtime Health

**Feature:** 011 | **Status:** DRAFT - do not execute before SPEC is ACCEPTED/LOCKED.
**Spec revision:** 0.2

## 011-T01 - Resolve GA OS/IPC and enabled first-slice groups
- **Status:** BLOCKED
- **Depends on:** NONE
- **Owner:** Product, Runtime, Operator IPC
- **Allowed paths:** this package and owner docs with approval
- **Contract:** SPEC §5; LOCAL-OPERATOR-IPC; RUNTIME-LIFECYCLE
- **Change intent:** Resolve B-28/B-29: declared GA desktop OS/versions and first served Settings category subset.
- **Acceptance:** OS/Runtime qualification matrix, authenticated peer tests and explicit list of interactive Settings controls whose owner route is served. All other target operations remain unavailable or omitted.
- **Stop:** no native OS action is marked GA from WSL-only tests.
**Command:** `python3 .specify/scripts/validate_feature_package.py`

## 011-T02 - Authenticated IPC gate
- **Status:** SOURCE IMPLEMENTED / PLATFORM AND PACKAGED-PEER QUALIFICATION OPEN
- **Depends on:** T01
- **Owner:** Operator IPC
- **Allowed paths:** IPC adapters and tests
- **Contract:** LOCAL-OPERATOR-IPC
- **Change intent:** Reject unauthenticated peers on each qualified OS.
- **Acceptance:** AC-001, AC-002, AC-009, AC-010; no bearer fallback, a failed Windows DACL does not open an unauthenticated pipe, remote peers are rejected, redacted frames, OS security tests.
- **Command:** `cargo test -p operator-ipc --locked ipc_peer_auth`

## 011-T03 - Runtime health and power/recovery observation
- **Status:** PARTIAL SOURCE / OS PROVIDER NOT QUALIFIED
- **Depends on:** T01
- **Owner:** Runtime / native desktop bridge
- **Allowed paths:** runtime lifecycle, Tauri OS adapters, tests
- **Contract:** RUNTIME-LIFECYCLE and SPEC Notifications/Advanced sections
- **Change intent:** Expose separate process incarnation/readiness/drain and qualified keep-awake assertion state; keep startup policy independent from resume policy.
- **Acceptance:** AC-005, AC-006, AC-012 plus native power assertion tests on declared OS; graceful drain stops admission before DRAINING and does not claim STOPPED before admitted handlers finish; unsupported battery/power modes are unavailable; resume only for durable LiteCowork-owned work and never all CLI process state.
- **Current gap:** Lifecycle reads/start/refresh exist; no evidenced keep-awake power provider or generic native-agent resume implementation.
- **Stop:** never report an OS assertion or resumed child process from UI-only state.
**Command:** `python3 .specify/scripts/validate_feature_package.py`

## 011-T04 - Settings semantic gate and common UI
- **Status:** PARTIAL SOURCE / NOT RELEASE QUALIFIED
- **Depends on:** T01–T03
- **Owner:** Desktop UI
- **Allowed paths:** Settings shell and panel components
- **Contract:** V1-UI-SCREEN-CONTRACT.md, V1-SETTINGS-CONTRACT-LEDGER.md, SEMANTIC-SERVED-UI-MAP
- **Change intent:** Align actual SettingsView with the chosen IA and render each control according to its owner class.
- **Acceptance:** AC-003, AC-004, AC-007, AC-008, AC-011; the mounted four-category SettingsView is mapped control-by-control (B-33 acceptance surface); local preferences actually apply and persist or remain unavailable; shortcut labels match registered handlers; no target-only category/control is presented as active; categories and disabled-control explanations are keyboard- and screen-reader-accessible; the Notifications entry links to Feature 010's quiet-hours contract and does not republish it; Runtime transport/readiness copy is accurate; all visible panels have empty/stale/offline/denied/error states. Do not count the unmounted SettingsPage or passed-but-unused props as controls.
- **Command:** `python3 .specify/scripts/validate_feature_package.py specs/011-desktop-shell-ipc-settings-runtime`
- **Stop:** no target-only button is rendered active.

## 011-T05 - Usage telemetry owner chain
- **Status:** NOT STARTED / TARGET ONLY
- **Depends on:** T01, T04
- **Owner:** Observability, Storage, Agent Runtime, Desktop UI
- **Allowed paths:** telemetry domain/schema/storage/OpenAPI/Operator/Tauri/UI/tests
- **Contract:** V1-TELEMETRY-USAGE-RESILIENCE-LLD.md
- **Change intent:** Implement per-inference leaf accounting and separate quota snapshot ledger, then expose scoped dashboard projections.
- **Acceptance:** source-only readers; immutable price snapshots; provider/native/gateway/estimate/unknown confidence; verified dedupe; exact Workspace/agent/model filters; DST-safe quick ranges and All; no double-count; no prompts/transcripts; redacted export. Full TLM adversarial test suite.
- **Stop:** no totals UI until both accounting stores + projections + route/command + generated types + tests are present.
**Command:** `python3 .specify/scripts/validate_feature_package.py`

## 011-T06 - Settings per-tab E2E and accessibility acceptance
- **Status:** NOT STARTED
- **Depends on:** T01–T05 and each domain owner feature accepted
- **Owner:** Desktop UI / QA / Product
- **Allowed paths:** settings UI and test harness
- **Contract:** SPEC, V1-UI-SCREEN-CONTRACT, Settings control ledger
- **Change intent:** Execute every shipped interactive control against real owner behavior on exact packaged desktop build/OS.
- **Acceptance:** every action has a working owner receipt/observation, all keyboard/focus/zoom/contrast/reduced-motion states; no dead button; screenshot snapshots and real OS tests.
- **Stop:** target features without owners remain hidden or disabled with reason.


**Command:** `python3 .specify/scripts/validate_feature_package.py`

