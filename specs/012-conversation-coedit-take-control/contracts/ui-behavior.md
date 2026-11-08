# UI behavior - Conversation co-edit and take-control

Behavior, not pixels. Screen inventory: [V1-UI-SCREEN-CONTRACT.md](../../../docs/V1-UI-SCREEN-CONTRACT.md). Control classes: [V1-SETTINGS-CONTRACT-LEDGER.md](../../../docs/V1-SETTINGS-CONTRACT-LEDGER.md) and [SEMANTIC-SERVED-UI-MAP.md](../../../docs/SEMANTIC-SERVED-UI-MAP.md). Action states: [V1-UI-ACTION-STATE-LEDGER.md](../../../docs/V1-UI-ACTION-STATE-LEDGER.md).

## Live flow anchors

Co-edit is a presentation over the ordinary Conversation flows in [FLOWS.md](../../../docs/FLOWS.md) (F00-F143); it cites live flow IDs, never the retired `UIV1-xxx` namespace (register [A-39](../../../docs/CONTRADICTION-AND-ALIGNMENT-REGISTER.md), provenance-only).

- **F01** (Conversation messages only; ordinary exchange, no Task card) - the baseline co-edit lives on; take-control returns the Conversation to this ordinary owner-authored exchange and the side-note is ordinary history, never a Task card.
- **F131** (Compose optional rich Conversation response) and **F134** (Stream and fence rich draft frames) - the driver indicator and side-note render alongside bounded transient frames without blocking or crossing turns.
- **F141** (Switch composer Agent and rebuild dependent native surface) - the driver indicator is derived from committed turn status plus the latest control event, not from the selected agent in the picker.
- **F142** (Native slash command or @ reference in chat) - side-chat is an ordinary owner message distinct from native namespaces and is revalidated at commit; it never enters the running turn's native prompt.

## Target flows and states

- A driver indicator shows, truthfully, whether an agent turn is running, the owner is in control, or a side-chat is visible. It is sourced from committed turn status and the latest `OWNER_TOOK_CONTROL` / `OWNER_RELEASED_CONTROL` control event, never inferred from the selected agent. Verifies AC-012.
- The take-control affordance is offered only while a turn is non-terminal. Invoking it issues the Feature 001 cancel path and shows an honest "stopping" state until native quiescence is observed; it does not claim the turn is cancelled before settlement. Verifies AC-001.
- When take-control settles, the Conversation moves to owner-in-control with the stopped turn's committed content preserved as editable context; the owner continues by hand and a later message admits a new ordinary turn. Verifies AC-002.
- If a completion/failure wins the race, the UI shows that terminal outcome, not a fabricated cancellation. Verifies AC-010.
- The side-chat composer posts a bounded `OWNER_SIDE_NOTE` correlated to the running turn; it renders with its correlated turn id and does not mutate or reroute the turn. An over-budget or terminal-turn note is rejected with its exact reason and the draft is preserved. Verifies AC-003, AC-005, AC-006.
- No co-edit affordance exposes the native session handle, private transcript or credentials; nothing implies hidden native memory was transferred. Verifies AC-008.
- Neither affordance opens a second concurrent active turn or creates a Task, Attempt, Coworker, lease or delegated execution. Verifies AC-009.

## Current implementation crosswalk (not the target)

- The mounted `ConversationsPage` has a Send/Stop composer, but Stop is ineffective for the OpenCode path and searches transient turns that path never emits (register A-25). There is no take-control, side-chat or driver indicator today.
- The transcript renders committed messages only; there is no `OWNER_SIDE_NOTE` kind, no control-event projection and no owner-in-control state.
- The supervisor dispatches OpenCode only; take-control reuses its cancel/quiescence authority rather than adding a provider path.

## Required UI regression cases after implementation

- Take-control on a running turn: honest stopping state, settlement only after quiescence, owner-in-control state with preserved content, and a truthful driver indicator throughout.
- Take-control when a completion/failure wins the race: terminal outcome shown, no fabricated cancellation.
- Side-chat on a running turn: correlated note rendered; the running turn's answer is unchanged and the note is absent from the native prompt.
- Side-chat on a terminal turn or over budget: rejected with the exact reason; draft preserved.
- History reload after take-control and side-chat: driver indicator and side-note correlation remain bound to committed records, not the currently selected agent; accessible state labels present.

## Validation

Validation must distinguish TARGET from the current partial UI. Source-level helper tests do not prove a rendered flow. Success requires a committed turn settlement, an immutable control event or correlated side-note, and a driver indicator rendered from that committed state.
