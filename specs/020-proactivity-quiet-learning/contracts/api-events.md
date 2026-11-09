# API and Event Contracts - Feature 020

**Status:** IN_REVIEW - not owner-accepted. Served routes are listed as they exist; every other interface is PROPOSED in design and not in source.
**Canonical owners:** [API](../../../docs/API.md), [Events](../../../docs/EVENTS.md), [Responsibilities](../../../docs/RESPONSIBILITIES.md), [Learning](../../../docs/LEARNING.md).

## 1. Served routes (CODE/SERVED today)

| Route | Handler | Notes |
|---|---|---|
| `GET /v1/suggestions?status=&visibility=&cursor=&limit=` | `suggestions_operator.rs` `list_suggestions` | Settles due expiries first (batch 100) |
| `POST /v1/suggestions/{suggestion_id}/resolve` | `resolve_suggestion` | `If-Match`, `Idempotency-Key`; DISMISSED by owner |
| `POST /v1/suggestions/{suggestion_id}/accept-task` | `accept_suggestion_task` | Atomic READY Task + ACCEPTED; returns `SuggestionTaskAcceptanceReceipt` |
| `POST /v1/suggestions/{suggestion_id}/snooze` | `snooze_suggestion` | `snoozed_until` set or cleared |
| `GET /v1/workspaces/{workspace_id}/suggestion-preferences` | `list_suggestion_preferences` | Includes virtual defaults |
| `PUT /v1/workspaces/{workspace_id}/suggestion-preferences/{kind}` | `set_suggestion_preference` | `If-Match: 0` for virtual default |
| `GET/POST /v1/goals`, `GET /v1/goals/{goal_id}`, `POST .../revisions`, `POST .../status` | `goal_operator.rs` | Intent and references only |
| `GET /v1/needs-you` | `needs_you_operator.rs` | Read-only; `Cache-Control: no-store` |

## 2. Proposed routes (LEARNING section 18; promoted by T02)

| Route | Purpose | Rules |
|---|---|---|
| `GET /v1/coworkers/{id}/learning-candidates?state=&kind=&cursor=&limit=` | Review list | Bounded preview only; `no-store` |
| `GET /v1/coworkers/{id}/learning-candidates/{candidate_id}` | Detail | Source ids, reason codes, redacted preview |
| `POST /v1/coworkers/{id}/learning-candidates/{candidate_id}/approve` | Approve NEEDS_REVIEW | `If-Match`, `Idempotency-Key`; Routine/Automation kinds return an editor link and enable nothing |
| `POST /v1/coworkers/{id}/learning-candidates/{candidate_id}/dismiss` | Dismiss | Same headers |
| `GET /v1/coworkers/{id}/learning-policy` | Read policy | Missing row returns REVIEW_ALL default with `version: 0` |
| `POST /v1/coworkers/{id}/learning-policy/revisions` | Revise policy | Rejects `AUTO_CONFIRMED_AND_LOW_RISK_INFERRED_PRIVATE` in V1 |
| `GET /v1/coworkers/{id}/memory?cursor=&limit=` | Learned memory list | ACTIVE and recently REVOKED |
| `POST /v1/coworkers/{id}/memory/{memory_id}/revoke` | Revoke or Undo | `If-Match`, `Idempotency-Key` |
| `POST /v1/conversations/{conversation_id}/messages/{message_id}/feedback` | Remember this, Correct | Owner's own committed message in a Coworker Conversation only |

## 3. Internal service contract (PROPOSED)

`SuggestionService::propose(SuggestionCandidate) -> ProposeOutcome` with outcomes `Created`, `SuppressedMuted`, `SuppressedCooldown`, `SuppressedDuplicate`, `SuppressedBudget`, `SuppressedScope`. Not exposed as an Operator route: only registered producers inside the daemon call it.

## 4. Events

Existing schema entries: `suggestion.proposed.v1` (first producer is T04/T05), `suggestion.resolved.v1`, `suggestion.visibility.changed.v1`, `suggestion.preference.changed.v1`, `goal.created.v1`, `goal.revised.v1`, `goal.status.changed.v1`.

Proposed (T02): `learning.candidate.created.v1`, `learning.candidate.status.changed.v1`, `learning.memory.committed.v1`, `learning.memory.revoked.v1`, `coworker.learning_policy.revised.v1`. Payloads contain ids, digests, reason codes, stable disposition tokens and bounded metadata; the schema rejects body, text or value fields.

## 5. Error codes (PROPOSED names; promoted by T02)

| Code | When |
|---|---|
| `LEARNING_POLICY_MODE_UNSUPPORTED` | Inferred auto mode requested in V1 |
| `LEARNING_CANDIDATE_STALE` | `If-Match` mismatch or candidate already terminal |
| `COWORKER_NOT_ACTIVE` | Paused or archived Coworker for a learning mutation or scoped proposal |
| `LEARNING_SOURCE_INELIGIBLE` | Feedback on a message that is not the owner's, not committed, TEMPORARY or not in a Coworker Conversation |

Existing `SUGGESTION_EXPIRED` and the current stale-version and idempotency codes keep their meaning.
