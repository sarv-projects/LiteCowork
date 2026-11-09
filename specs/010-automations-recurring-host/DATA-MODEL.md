# Data model - Automations and Recurring Host

**Feature:** 010 | **Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.

Identities and lifecycles stay in the canonical owner documents. This file lists the records this slice reads or commits. It does not authorize new tables by itself.

## Records

- Automation revision.
- Occurrence id and cursor.
- Task link.
- Notification status.

## Relations

Workspace is the security boundary. Child records carry workspace id. Runtime-local secrets, handles and private locators are excluded from replicated events and ordinary backups.

## Lifecycle

Create and settle through the transactions in WIRING.yaml. External effects happen after commit. Terminal states are explicit. Unknown is not success.

## Persistence and events

Use existing event names in [docs/EVENTS.md](../../docs/EVENTS.md). A new event type requires a same-change edit to EVENTS.md and `docs/schemas/domain-event.schema.json`.

## Retention

Follow the owner retention rules. Purge removes future visibility of payload bytes. Receipts of the deletion may remain.

## Migration

No migration has shipped for this package yet. B-26 and B-27 are accepted (2026-11-09); the delivery lease/status transition columns, SUPPRESSED delivery rows and the event-trigger cursor land as the next immutable `docs/schemas/sqlite-vNN.sql` through TASKS.md T01, T06, T07 and T09.


## External workflow references (provider-owned)

Feature 010 does not create Workflow, WorkflowNode, WorkflowEdge, WorkflowRun or WorkflowStep tables. A future qualified adapter may define a provider-scoped workflow reference and normalized input/output mapping for a Routine; no canonical WorkflowRef machine type or production provider adapter currently exists in this checkout. The external graph, revision, node state and provider run identity remain provider-owned; runtime-only external run handles are protected as provider-private data. The LiteCowork CapabilityInvocation and Task/Attempt receipts remain the local durable authority.

No migration is proposed for this reference comparison. A new external authoring capability requires a separate owner decision and provider contract.
