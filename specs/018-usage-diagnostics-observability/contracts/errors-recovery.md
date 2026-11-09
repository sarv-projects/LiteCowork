# Error and Recovery Contract - Feature 018

**Status:** IN_REVIEW - pending owner acceptance of B-42; does not authorize implementation; not a claim of implementation or qualification.
**Owners:** Observability, Operator/API, Security, Desktop UI.
**Canonical owners:** [Failure Recovery](../../../docs/FAILURE-RECOVERY.md), [Observability](../../../docs/OBSERVABILITY.md), [Telemetry LLD](../../../docs/V1-TELEMETRY-USAGE-RESILIENCE-LLD.md), [API](../../../docs/API.md).

## 1. Error classes (PROPOSED codes promoted by 018-T02)

| Code | HTTP | Cause | Server action | Client action |
|---|---|---|---|---|
| `FORBIDDEN` / `NOT_FOUND` (existing) | 403 / 404 | Foreign principal or Workspace; All Workspaces without selector | Deny; reveal no counts | Do not retry unchanged |
| `INVALID_ARGUMENT` (existing) | 400 | Bad id format | Deny | Fix input |
| `USAGE_WINDOW_INVALID` | 400 | start not before end, more than 400 buckets, non-UTC timestamp | Deny | Recompute the window |
| `SUPPORT_BUNDLE_TOO_LARGE` | 422 | Selection above the cap | Write nothing | Deselect categories |
| `DIAGNOSTICS_UNAVAILABLE` | 503 | Diagnostics directory unreadable or full | No partial file | Show repair path |
| `IDEMPOTENCY_CONFLICT` (existing) | 409 | Same key, different bundle or export payload | No write | New key for new intent |
| `DEPENDENCY_UNAVAILABLE` (existing) | 503 | Store unavailable | No partial result | Retry later |

## 2. Recovery rules

- **Missing telemetry:** never an error. Missing or malformed counters become UNKNOWN or ESTIMATED_LOCAL; Feature 001 settlement proceeds.
- **Replay:** a replayed final commit writes no second usage row (record id derived from turn, retry ordinal and response id).
- **Restart:** projections are recomputed from records; the summary is identical before and after restart.
- **Log backpressure:** a full channel drops debug entries and increments a dropped counter; it never blocks admission, settlement or Effects.
- **Disk pressure:** logs stop writing to file, keep the in-memory ring, and the health probe reports DEGRADED; no silent deletion of domain data.
- **Crash during bundle or export write:** the file is created with create-new and renamed or fsynced only when complete; an incomplete file is removed at next start and never reported as written.
- **Crash hook failure:** if the crash record cannot be written, the default hook still runs; the next start reports "crash record unavailable" rather than inventing one.
- **Stale health:** a probe older than its cutoff is UNKNOWN, never READY.
- **Quota producer absent or expired:** show "Not reported by this agent" or Unknown with reason.

## 3. Privacy failure is a release blocker

Any canary (prompt, completion, token, cookie, handle, Resource path, passphrase) found in a usage row, log entry, crash record, export or bundle fails the task and blocks qualification on every row.
