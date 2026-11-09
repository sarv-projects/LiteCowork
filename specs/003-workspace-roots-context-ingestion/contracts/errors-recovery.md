# Errors and recovery - Workspace Roots and Context Ingestion

**Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.

UNAVAILABLE, PAUSED and REVOKED are distinct. Stale Workspace version conflicts without partial writes. Identity mismatch on resume leaves the root PAUSED. Lost keyring identity is a startup blocker and must not be reported as a successful scan.

A root paused or revoked between ResourceRef selection and turn dispatch fails the attachment with a typed root-not-active error and no bytes reach the agent (AC-019). A crash after the retrieval receipt commit but before dispatch reloads the receipt and does not re-run retrieval with a different result for the same turn. On Windows, a reparse point, UNC, device or alternate-data-stream path fails closed with no partial binding.

Denial, duplicate replay, timeout-after-dispatch, cancel, crash and restart are all required before QUALIFIED. Unknown external results reconcile before retry.
