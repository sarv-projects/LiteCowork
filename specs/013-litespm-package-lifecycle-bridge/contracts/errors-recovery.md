# Error and Recovery Contract - Feature 013

**Status:** IN_REVIEW - not owner-accepted; does not authorize implementation; not a claim of implementation or qualification.
**Owners:** Capability Fabric, Storage, Security, Desktop UI.
**Canonical owners:** [Failure Recovery](../../../docs/FAILURE-RECOVERY.md), [interop LLD section 6](../../../docs/LITESPM-LITECOWORK-INTEROP-LLD.md), [API](../../../docs/API.md).

## 1. Classification and client rule

| Class | Examples | Server action | Client action |
|---|---|---|---|
| Owner/scope denial | foreign principal, foreign Workspace, `RUNTIME_MISMATCH`, archived Workspace | deny before any LiteSPM call | show denial; no retry |
| Service not qualified | B-36/B-37 not accepted, row not qualified | 503 `LITESPM_PACKAGE_SERVICE_NOT_QUALIFIED` | verb disabled with reason |
| Protocol mismatch | unpinned version, missing descriptor, unadvertised op | `LITESPM_PROTOCOL_INCOMPATIBLE` / `LITESPM_OPERATION_NOT_ADVERTISED` | show update-required or unavailable |
| Peer refused | endpoint owner differs from current user | refuse connection; record `REFUSED` | show service unavailable; no fallback |
| Inventory unavailable | LiteSPM read failed or timed out | `PACKAGE_INVENTORY_UNAVAILABLE`; never empty page | show unavailable with Retry |
| Profile unsupported | project scope, `mcp`, unknown host version | `PACKAGE_HOST_PROFILE_UNSUPPORTED` | verb disabled with reason |
| Plan stale | expired plan, changed fingerprint | `PACKAGE_PLAN_EXPIRED` | re-plan; old approval unusable |
| Approval conflict | consumed approval, replay with changed payload | `PACKAGE_APPROVAL_CONSUMED` / `IDEMPOTENCY_CONFLICT` | refresh receipt |
| Target busy | another receipt `DISPATCHING`/`UNKNOWN` for the target | `PACKAGE_OPERATION_UNKNOWN_PENDING` | show Reconcile first |
| Apply uncertain | timeout or lost response after request write | settle `UNKNOWN`; query operation identity | show "Outcome unknown, checking" |
| LiteSPM terminal failure | precondition changed, write refused | settle `FAILED` with LiteSPM code | show cause; offer re-plan or rollback plan |
| Drift | host entry edited outside LiteSPM | read-only diff; keep-local default | owner chooses resolution |

## 2. Retry classes

- **SAFE_READ_RETRY:** status, inventory, describe, operation get.
- **SAFE_LOCAL_REPLAY:** same principal, key and payload for approval or apply returns the stored record.
- **RECONCILE_BEFORE_RETRY:** any apply in `DISPATCHING` or `UNKNOWN`; never re-send apply.
- **NEW_INTENT_ONLY:** a changed plan, scope or host requires a new plan and approval.

## 3. Crash windows

| Window | Recovery |
|---|---|
| After approval commit, before apply | Approval stays GRANTED until expiry; no mutation happened |
| After `begin_apply` commit, before request write | Startup sweep queries operation by apply key; absent upstream record means settle FAILED (not dispatched) only when LiteSPM proves the key unknown |
| After request write, before response | `UNKNOWN`; query original operation identity |
| After LiteSPM success, before `settle_apply` | Query returns terminal success; settle with read-back |
| LiteSPM restart mid-apply | LiteSPM's own journal recovers; LiteCowork queries after reconnect |
| UI closed during apply | Receipt state survives; UI rehydrates from `GET /v1/package-operations/{receipt_id}` |

## 4. Never

Never report success from a timeout, never repeat an apply to "fix" an UNKNOWN, never restore host files, never treat a transport-ACL-only result as verified identity, never treat an empty failed read as "nothing installed".
