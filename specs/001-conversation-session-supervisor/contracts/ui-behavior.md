# UI behavior — Conversation Send and turn progress (Feature 001)

**Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification. Binds UI to owner contracts; not pixels.  
**Normative UI indexes:** [V1-UI-SCREEN-CONTRACT.md](../../../docs/V1-UI-SCREEN-CONTRACT.md), [V1-UI-ACTION-STATE-LEDGER.md](../../../docs/V1-UI-ACTION-STATE-LEDGER.md) (UIV1-016–022, 049–057), [DESKTOP-SHELL-LLD.md](../../../docs/DESKTOP-SHELL-LLD.md), [NATIVE-COMPOSER-LLD.md](../../../docs/NATIVE-COMPOSER-LLD.md).  
**Wire:** [LOCAL-OPERATOR-IPC.md](../../../docs/LOCAL-OPERATOR-IPC.md) → OpenAPI in [api-events.md](api-events.md).

UI **renders** conversation projections and stream frames; **sends** typed Operator commands. It does not parse English error strings.

## 1. User flow and transitions

| Step | Screen / surface | Transition |
|---|---|---|
| F1 | Conversation route, composer visible | User has eligible binding or sees repair state |
| F2 | User edits draft | Local reducer only; no domain mutation |
| F3 | Send | Admission command → turn OPEN/RUNNING/WAITING_* per state machine |
| F4 | Streaming | Stream frames + projection reconcile; scroll policy per shell LLD |
| F5 | Stop | Cancel command; terminal turn settlement |
| F6 | Failure / ambiguous | Typed error + repair; draft preserved where specified |
| F7 | Reload / restart app | Read models + RESYNC_REQUIRED if stream gap |

No navigation to Task or Coworker is required for F1–F7.

## 2. UI states (composer and message stream)

| Surface | States | User-visible meaning |
|---|---|---|
| Composer eligibility | `EMPTY`, `NEEDS_AGENT`, `AUTH_REQUIRED`, `ELIGIBLE`, `SENDING`, `WAITING`, `AMBIGUOUS`, `SETTLED`, `OFFLINE`, `STALE` | From binding descriptor + turn coordinator; disabled control **must** show reason ([V1-UI-ACTION-STATE-LEDGER.md](../../../docs/V1-UI-ACTION-STATE-LEDGER.md)) |
| Agent picker | `LOADING`, `READY`, `AUTH_REQUIRED`, `STALE` | Atomic refresh of model/mode/effort; draft text retained |
| Message stream | `LOADING`, `STREAMING`, `COMMITTED`, `DEGRADED`, `ERROR`, `PARTIAL` | Semantic message authoritative; rich draft optional and droppable |
| Runtime / admission | `UNAVAILABLE`, `DEGRADED` | Serving daemon ≠ qualified supervisor; fail-closed until feature verified |

## 3. Validation and failure presentation

| Validation | Rule | On failure (typed, not string parsing) |
|---|---|---|
| Attachments | Resource pins + agent descriptor support at send time | `UNSUPPORTED`, `STALE`, `DENIED` UI states; show repair link to context picker |
| Send while prior turn active | B-01 is LOCKED: strict single-flight; reject a concurrent submit with CONVERSATION_TURN_IN_PROGRESS (HTTP 409), preserve the draft and show the active-turn state | Operation error code + ledger state |
| Binding unavailable | No send; preserve draft | `NEEDS_AGENT` / `AUTH_REQUIRED` |
| Idempotency replay | Same `RequestId` returns same outcome or `AMBIGUOUS` lock | UI locks retry until reconcile ([errors-recovery.md](errors-recovery.md)) |
| UserRequest response | Immutable response + exact provider input | UIV1-140–147; AC-017–019 |

## 4. Commands (UI → core)

Map each control to **OpenAPI operationId** (see [api-events.md](api-events.md)). Transport: Tauri → [LOCAL-OPERATOR-IPC.md](../../../docs/LOCAL-OPERATOR-IPC.md) → same handlers as HTTP Operator.

| UI control (ledger) | Reducer action (local) | Domain command (typed) |
|---|---|---|
| Send | `TurnAdmissionRequested` / `TurnRequested` | Conversation message submit / turn admission (exact operationId in api-events) |
| Stop | `CancelRequested` | Turn cancel (owner API) |
| Retry | `RetryRequested` | Idempotent retry per owner rules |
| Agent change | `SelectedBindingChanged` | Binding read + descriptor refresh; send uses new binding only on new admission |
| UserRequest approve/deny | `UserRequestResponded` | UserRequest response commit (owner API) |

Local-only: draft text, scroll anchor, IME state — not persisted as domain events.

## 5. Events and stream (core → UI)

| Source | Type | UI use |
|---|---|---|
| Operator stream | `OperatorStreamFrame` variants ([PRESENTATION-RUNTIME.md](../../../docs/PRESENTATION-RUNTIME.md)) | Live token/draft/settlement; gap → `RESYNC_REQUIRED` |
| Read APIs | Conversation + turn projections | Hydration after navigation/restart |
| Domain events | e.g. `conversation.message.added.v1`, `conversation.turn.settled.v1` | Not consumed directly in WebView; materialize via projection/stream |

## 6. Error shapes

Use [SCHEMAS.md](../../../docs/SCHEMAS.md) and `schemas/error-codes.schema.json`. UI maps:

- HTTP/OpenAPI error body → toast/banner + control state
- Admission `failure_code` → composer eligibility
- `AMBIGUOUS` → pending lock UI (no duplicate send)

## 7. Tests (behavior IDs)

| ID | Scope |
|---|---|
| UIV1-016–022 | Composer send/stop/retry/eligibility |
| UIV1-023–036 | Agent picker + attachments + native palette |
| UIV1-049–057 | Stream reconcile, degraded rich presentation |
| AC-001–019 | Feature EARS in [SPEC.md](../SPEC.md) |
| FL01–FL05 | [V1-END-TO-END-FLOW-AND-RACE-CONTRACTS.md](../../../docs/V1-END-TO-END-FLOW-AND-RACE-CONTRACTS.md) |

System/USER qualification requires the packaged desktop, daemon and exact real harness route ([ACCEPTANCE-AND-E2E-PLAN.md](../../../docs/ACCEPTANCE-AND-E2E-PLAN.md)); it is not established for this checkout.

## 8. Current source crosswalk — 2026-11-06

- The mounted Conversation surface is ConversationsPage.tsx. Recent Chats and message history are cursor-paginated; Workspace/list-epoch fencing, conversation-ID deduplication, local per-Conversation drafts and the unreadable pending-create journal pause/retry are implemented.
- The active-turn projection exposes status only, not aggregate version. The composer treats only RUNNING and presentation-stream transient turns as running; OPEN, WAITING_DEPENDENCY and CANCEL_REQUESTED are not consistently fenced. A 202 response can clear the local sending flag while the durable turn remains OPEN. The backend submit transaction does not enforce B-01 strict single-flight.
- Stop is shown for RUNNING, but the handler searches only the presentation-stream transient-turn collection. The OpenCode Operator path does not emit those turns. If one exists, the call hardcodes expectedVersion=1. The Operator cancel route transitions RUNNING to CANCEL_REQUESTED and immediately to CANCELLED without a native interrupt or observed quiescence.
- The Operator retry route exists, but the UI has no retry call site or failed-turn affordance. The route transitions FAILED to RUNNING with a new session pointer before calling a dispatcher reservation that requires OPEN and no session pointer; retry is not integrated correctly.
- The agent-binding selector and Send are wired. Send now enforces the disabled-until-qualified UI gate: it requires a qualified (selectable) AgentBinding, a non-empty draft, Operator readiness and no in-flight turn (`sendEligible` predicate; register A-24 resolved). This UI gate is not full OS/provider/auth/USER qualification — that remains the Feature 001 G-09 target. The Add context/file and Voice Input buttons and More options button have no handlers; Turbo is static; the @/slash composer copy has no mention/action implementation. Submit always sends resource_refs=[]; there is no attachment picker or Context Used UI.
- The empty Conversation canvas now renders truthful, LiteCowork-branded copy (no third-party Antigravity branding, no unsupported capability claims; register A-26 branding resolved), guarded by `apps/litecowork-ui/tests/conversation-empty-state-truthfulness.test.mjs`.
- Production ConversationApiState::from_ref constructs AgentSessionSupervisor and submit/retry call dispatch_turn. The module header in conversation_operator.rs still says dispatch is intentionally absent; that comment is stale. The actual dispatch branch is OpenCode-only and remains unqualified.
- See [IMPLEMENTATION-STATUS.md](../../../docs/IMPLEMENTATION-STATUS.md), [SETTINGS-AND-SHELL-SURFACE-AUDIT.md](../../../docs/SETTINGS-AND-SHELL-SURFACE-AUDIT.md) and Feature 001 SPEC for the current source/owner crosswalk.

## 9. Cross-layer trace per user-visible operation (001-T21, 2026-11-10)

Chain per row: outcome, exact control, typed UI command, authenticated Operator route, authorization, service owner, transaction/version/fence, aggregate/event/receipt, projection/stream, restart/recovery, evidence. Evidence class is CODE unless stated; nothing here is QUALIFIED (T09 needs a live release-matrix run). All Tauri commands are in `src-tauri/src/conversation_bridge.rs` and cannot be compiled in the Linux container (gdk-3.0 missing).

| Operation | Control and command | Route and authorization | Owner, transaction, fence | Event, projection, recovery | Evidence |
|---|---|---|---|---|---|
| Send | Send button or Enter in the composer (`sendTurn`) -> `submit_conversation_turn` | `POST /v1/conversations/{id}/turns`; authenticated local peer, `X-Workspace-ID`, `Idempotency-Key`; Conversation must belong to the owner and Workspace; attachments reauthorized (`check_turn_resources`) | `AgentTurnCoordinator` after one transaction that commits the USER message, OPEN turn, dispatch intent and receipt; B-01 single-flight returns 409 `CONVERSATION_TURN_IN_PROGRESS` | `conversation.message.added.v1`; presentation snapshot and `turn.delta`/`turn.settled` frames; the durable dispatch intent is claimed once and `recover_conversation_sessions` runs at startup | `conversation_turn_submit_*`, `conversation_turn_delta_*`, `conversation_usage_*` |
| Stop | "Stop responding" button (`cancelTurn`) -> `cancel_conversation_turn` with the projected turn id and version | `POST /v1/conversations/{id}/turns/{turn_id}/cancel`, `If-Match` = aggregate version, `Idempotency-Key` | Supervisor `request_cancel` signals the attempt; the turn moves to CANCEL_REQUESTED, and CANCELLED is committed only after observed quiescence | `conversation.turn.status.changed.v1`, `conversation.turn.settled.v1`; an unconfirmed stop stays CANCEL_REQUESTED and is reconciled at restart | `conversation_cancel_commits_only_the_request_and_never_fabricates_cancelled`, `conversation_stop_signal_reaches_only_the_registered_attempt_and_is_released` |
| Retry | Retry button in the inline FAILED card (`retryTurn`) -> `retry_conversation_turn` | `POST /v1/conversations/{id}/turns/{turn_id}/retry`, `If-Match`, `Idempotency-Key` | FAILED -> OPEN with `retry_ordinal` + 1 and one new intent, no second USER message; stale `If-Match` is 412, a non-FAILED turn 409 | New attempt streams under its own ordinal; stale ordinals are dropped by the reducer | `conversation_stop_and_retry_failed_turn_lifecycle`, `conversation_retry_is_refused_while_another_turn_is_in_flight` |
| Reload | Opening or re-selecting a Conversation, `Load earlier messages`, stream reconnect -> `get_conversation_presentation`, `subscribe_conversation_stream` | `GET /v1/conversations/{id}/presentation`; authenticated stream with Workspace and Conversation scope | Read-only; cursor bound to Workspace and Conversation | Committed messages are authoritative; a sequence gap or `stream.resync_required` drops transient text and re-reads; a restarted app rebuilds from SQLite | presentation snapshot tests, `conversation-stream.test.ts`, `presentation-stream.test.ts` |
| Activity | Collapsed "Activity reported by the agent" card and plan line (`TurnActivityView`) -> `list_conversation_turn_activity` | `GET /v1/conversations/{id}/turns/{turn_id}/activity?retry_ordinal=` | Read-only; REPORTED records written by the supervisor's `TurnActivityRecorder` before each `turn.activity` frame | Not a DomainEvent; re-read for the active or failed turn after restart | `turn_activity_records_*` (daemon, storage), `turn-activity.test.ts` |
| Attach | "+" in the composer (`openPicker`) -> `list_resources`, then `submit_conversation_turn` with `resourceRefs` | Resource list route, then the Send route | `materialize_conversation_resources` reauthorizes each ref at dispatch; a refusal fails the turn before dispatch | Inlined text carries id, revision and digest; no ContextAttachment record yet (003-T09) | `conversation_resource_materialization_*`, `conversation-attachments.test.ts` |
| Answer UserRequest | **GAP, no hop exists.** Needs You answers UserRequests through `user_request_operator.rs`, but no Conversation turn creates a Conversation-scoped UserRequest because the OpenCode adapter surfaces no provider input request | `POST /v1/user-requests/{id}/respond` exists for other scopes | Domain WAITING_USER/WAITING_DEPENDENCY transitions exist, with no producer | none | open task: 001-T12 (BLOCKED) |
| Native permission | **PARTIAL.** `PermissionAsked` becomes a PERMISSION activity card worded "the agent asked for permission ... LiteCowork is not asking". No answer control: there is no verified answer channel (ACP `request_permission` is 002-T17; no OpenCode permission-reply capture) | none | none; never a LiteCowork grant or Effect | REPORTED record only | open task: 001-T27 (PARTIAL) |

