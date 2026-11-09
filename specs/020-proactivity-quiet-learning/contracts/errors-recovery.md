# Errors and Recovery Contract - Feature 020

**Status:** IN_REVIEW - not owner-accepted; authorizes no implementation.
**Owners:** Responsibility, Storage, Desktop UI, Security.
**Canonical owners:** [FAILURE-RECOVERY](../../../docs/FAILURE-RECOVERY.md), [LEARNING](../../../docs/LEARNING.md) section 21, [RESPONSIBILITIES](../../../docs/RESPONSIBILITIES.md).

## 1. Suggestion cases

| Case | Server action | Client action |
|---|---|---|
| Kind muted | SuppressedMuted; nothing stored | none (owner sees nothing new) |
| Same key dismissed in last 30 days | SuppressedCooldown | none |
| Same key already PROPOSED | SuppressedDuplicate | none |
| Budget reached | SuppressedBudget; reason counter | none |
| Coworker paused/archived or source no longer visible | SuppressedScope | none |
| Mute racing propose | One transaction wins; mute resolves open rows MUTED_KIND | Reload list |
| Stale version on owner action | Conflict; no write | Refresh card; keep the owner's choice for retry |
| Accept while source STALE/CONFLICTED | Opens review step; no silent advance | Show what changed |
| Producer crash before cursor advance | Re-run batch; propose dedupes | none |
| Clock jump past expiry | Service clock settles EXPIRED on next list | Card disappears with "Expired" in history |
| Quiet hours active | Proposal stored, `push_eligible=false` | No Home idea or notification; Ideas lists it |

## 2. Learning cases

| Case | Required behavior |
|---|---|
| Owner corrects memory while extraction in flight | Current source version wins; stale candidate SUPERSEDED |
| Same correction repeated | Dedupe by normalized key and source lineage |
| Inferred conflicts with confirmed | NEEDS_REVIEW |
| Secret detector hit or uncertain | REJECTED (forbidden) or NEEDS_REVIEW; never auto-commit |
| Payload Resource missing | REJECTED; never infer content from digest |
| Policy revised between evaluation and commit | Recheck fails; NEEDS_REVIEW with reason |
| Coworker paused or archived | No commit; history readable |
| Source message deleted (Feature 014) | Non-terminal candidates REJECTED |
| Classifier unavailable or timeout | Skip; increment skipped count; Conversation unaffected |
| Queue full | Bounded skip with reason; never block Send or settlement |
| Crash after Resource write, before candidate insert | Orphan Resource swept on restart |
| Crash between evaluation and commit | Candidate stays AUTO_COMMIT_ELIGIBLE; re-evaluated idempotently |
| External content says "remember this" | Ineligible channel; no candidate auto-commits |
| Owner says "do this every day" | AUTOMATION_PROPOSAL; NEEDS_REVIEW; nothing enabled |
| Owner says "never send without asking" | INSTRUCTION_PATCH or Trust proposal; NEEDS_REVIEW; no permission change |

## 3. Client rules

Preserve the owner's in-progress choice on conflict; never show "Remembered" before the commit event is projected; never show an Undo that cannot reach a served revoke route (render unavailable instead).
