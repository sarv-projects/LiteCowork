# UI behavior - Ideas, learning review and quiet memory (Feature 020)

**Status:** IN_REVIEW - not owner-accepted; authorizes no implementation. Binds UI to owner contracts; not pixels.
**Normative UI indexes:** [V1-UI-SCREEN-CONTRACT.md](../../../docs/V1-UI-SCREEN-CONTRACT.md), [V1-UI-ACTION-STATE-LEDGER.md](../../../docs/V1-UI-ACTION-STATE-LEDGER.md), [DESKTOP-SHELL-LLD.md](../../../docs/DESKTOP-SHELL-LLD.md).
**Wire:** [LOCAL-OPERATOR-IPC.md](../../../docs/LOCAL-OPERATOR-IPC.md) -> routes in [api-events.md](api-events.md).
**Hosts:** Feature 009 owns the Coworker view and onboarding; Feature 001 owns the Conversation view; Feature 010 owns the quiet-hours editor. This package supplies the components and slots named below.

UI renders projections and sends typed commands. It never decides eligibility, certainty or authority.

## 1. User flows

| Step | Surface | Transition |
|---|---|---|
| F1 See an idea | Home prominent idea (at most one) or Ideas page | Suggestion PROPOSED and `push_eligible` (Home) or any visible (Ideas) |
| F2 Accept a TASK idea | Ideas card Accept | READY Task created; open it in Work without starting it |
| F3 Accept an editor idea | Ideas card Open editor | Routine or Automation editor opens; nothing saved until Save |
| F4 Dismiss, snooze, unsnooze | Ideas card | DISMISSED (30-day cooldown) or hidden until time |
| F5 Mute a kind | Ideas preferences | Open ideas of that kind resolved MUTED_KIND |
| F6 Remember this | Message action on own message in a Coworker Conversation | Candidate created; commit or review per policy |
| F7 Quiet commit | Conversation notice "Remembered: ... [View]" | Shown after commit event; deferred during quiet hours |
| F8 Undo or Correct | Notice or memory list | Memory REVOKED or new CONFIRMED candidate |
| F9 Review | Coworker view > Learning > Needs review | Approve or Dismiss |
| F10 Policy | Coworker Settings > Learning, and onboarding | OFF, REVIEW_ALL, AUTO_CONFIRMED_PRIVATE |

## 2. UI states

| Surface | States | Meaning |
|---|---|---|
| Ideas list | LOADING, EMPTY, READY, ERROR, OFFLINE, STALE, DENIED | EMPTY says "No ideas right now"; OFFLINE says the local Runtime is unavailable and the view never starts work |
| Idea card | PROPOSED, SNOOZED, ACCEPTING, ACCEPTED, DISMISSED, EXPIRED, CONFLICT | CONFLICT offers Refresh and keeps the owner's choice |
| Home idea | HIDDEN (quiet hours or none), SHOWN | At most one |
| Learning list | LOADING, EMPTY, READY, ERROR, OFFLINE, DENIED, UNAVAILABLE | UNAVAILABLE until B-45 and routes ship; names the reason |
| Remembered notice | PENDING (not shown), SHOWN, UNDONE, CORRECTING | Never shown before commit is projected |
| Policy control | LOADING, READY, SAVING, CONFLICT, ERROR | Inferred auto mode is not offered |
| Memory list | LOADING, EMPTY, READY, REVOKING, ERROR | Revoked items labelled |

## 3. Card content rule

Each idea shows why it appeared, which pinned sources and Goal revision were considered, what acceptance will create (READY Task or editor), permissions that may be required, and the label "This does not start work".

## 4. Commands (UI -> core)

| Control | Command | Route |
|---|---|---|
| Accept (TASK) | `accept_suggestion_task` | `POST /v1/suggestions/{id}/accept-task` |
| Dismiss | `suggestion_owner_action("dismiss")` | `POST /v1/suggestions/{id}/resolve` |
| Snooze / Show now | `suggestion_owner_action("snooze" or "unsnooze")` | `POST /v1/suggestions/{id}/snooze` |
| Mute / Allow kind | `set_suggestion_preference` | `PUT /v1/workspaces/{id}/suggestion-preferences/{kind}` |
| Remember this / Correct | `record_message_feedback` (new) | `POST /v1/conversations/{cid}/messages/{mid}/feedback` |
| Approve / Dismiss candidate | `learning_candidate_action` (new) | `POST /v1/coworkers/{id}/learning-candidates/{cid}/approve` or `/dismiss` |
| Undo / Revoke | `revoke_learned_memory` (new) | `POST /v1/coworkers/{id}/memory/{mid}/revoke` |
| Change policy | `revise_learning_policy` (new) | `POST /v1/coworkers/{id}/learning-policy/revisions` |

## 5. Accessibility

Keyboard reachable controls with visible focus; card actions grouped with an accessible label; commit, undo and preference changes announced through a polite live region (the existing `SuggestionsPage` status region pattern); no modal for quiet commits; disabled or unavailable controls expose the reason; reduced motion honored; contrast WCAG 2.2 AA.

## 6. Cross-layer trace (completed by TASKS T14)

| Operation | Control | Command | Route | Fence | Receipt/event | Recovery | Evidence |
|---|---|---|---|---|---|---|---|
| Accept TASK idea | Ideas card Accept | `accept_suggestion_task` | accept-task | If-Match, idempotency | acceptance receipt, `suggestion.resolved.v1` | replay returns REPLAYED | T07, T15 |
| Remember this | Message action | `record_message_feedback` | feedback | owner, Coworker ACTIVE, idempotency | `learning.candidate.created.v1` | resume work item | T09, T11 |
| Undo | Notice Undo | `revoke_learned_memory` | memory revoke | If-Match | `learning.memory.revoked.v1` | context excludes after restart | T10, T11 |
