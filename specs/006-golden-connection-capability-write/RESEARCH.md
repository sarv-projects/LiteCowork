# Research - Golden Connection and One Capability Write

**Feature:** 006 | **Checked:** 2026-11-06 (source, composed routes and desktop UI); provider docs re-checked 2026-11-09

**Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.

## Sources

Authorities: docs/CONNECTORS-WORKFLOWS.md, docs/CAPABILITY-FABRIC.md, docs/NETWORK-SECURITY.md, docs/LITESPM-LITECOWORK-INTEROP-LLD.md. Reject embedding third-party automation runtimes. Upstream LiteSPM main was checked at `111fd9b8e84573f8fce76b6374b4ff9639502539` on 2026-11-05; its 13-tool local stdio bridge is WIRED upstream. LiteCowork's adapter remains TARGET / not qualified.

## Findings

- The owner documents already decide the trust boundaries this slice must keep.
- Upstream `STATUS.md` marks `internal/bridge` WIRED: `request_install` reaches supported skill/MCP/pinned-plugin installation paths; `get_invocation` and `cancel_invocation` reach the daemon invocation registry. These are upstream claims at the pinned commit, not LiteCowork integration proof.
- The bridge runs as local MCP stdio via `litespm bridge stdio --host <host-id>` and can cold-start/reconnect the daemon. Exact MCP tool schemas and host mapping still require a LiteCowork conformance fixture.
- Upstream's host UX is version-dependent: OpenCode v2.2.5 exposes `/litespm:marketplace`; Qwen Code 0.14.4 exposes tools through `/mcp` and does not use the same slash command.
- Windows peer identity is not independently verified by the current LiteSPM bridge (`peerAuth.verified=false`, owner-only Named Pipe DACL is the stated OS boundary). Do not label this application-level verified identity.
- Hosted competitors often collapse connection, permission and execution into one toggle. LiteCowork forbids that collapse.
- Reference desktop repositories are architecture comparisons, not code to vendor. Pins and paths: [REFERENCE-IMPLEMENTATIONS.md](../../docs/REFERENCE-IMPLEMENTATIONS.md), [REFERENCE-REPOS-CORE-ARCHITECTURE-INDEX.md](../../docs/REFERENCE-REPOS-CORE-ARCHITECTURE-INDEX.md).

## Borrowable-design sources (pinned)

| Source | Exact pinned URL | License | What informs the golden connection/write | Verdict |
|---|---|---|---|---|
| OpenWork lazy capability gateway (`search_capabilities`/`execute_capability`) | https://github.com/different-ai/openwork/tree/5493b18a6af64d56454658a53da00eabbc6e033d | MIT except `/ee` (Enterprise Edition License) | The lazy gateway admits a named capability only on demand, which maps to the broker loading only a grant-scoped invocation rather than exposing a standing provider surface. | Prototype the lazy-admission shape against CAPABILITY-FABRIC; do not import `/ee` paths or adopt the OpenCode loop. |
| LiteSPM published package/host contract | https://github.com/sarv-projects/LiteSPM/tree/9f42b25ddfb1acedce0f2dbf9964f4a7724d622e | Apache-2.0 at checked root LICENSE | LiteSPM is the selected package lifecycle authority; its published package/host contract is the boundary LiteCowork links to for install, not reimplements. The checked root does not yet settle the LiteCowork-facing wire contract, so no API is inferred. | Reference/boundary only; use the published contract when available, do not invent or duplicate its API. |
| Arcade MCP authorized tool declarations | https://github.com/ArcadeAI/arcade-mcp/blob/79a2dc90d347e4cd1a96366a2d94c8ddc356f1c3/README.md | MIT | Study authorized MCP tool declarations and secret-injection boundaries: tokens are injected by the host, never exposed to the model, which matches the SecretStore custody and token-redaction requirement (AC-001). | Reference only; Arcade MCP is an optional provider, not a required dependency of this slice. |

No OpenWork, LiteSPM or Arcade MCP code is imported; all three are prototype/reference/boundary inputs. OpenBot is not cited here; if it were, it would be reference/UX only with no code reuse under its PolyForm Noncommercial license.

## Source, composed-route and UI audit (2026-11-06)

Inspected `apps/litecoworkd/src/discover_operator.rs`, `capability_invocations_operator.rs`, `approval_operator.rs`, `trust_operator.rs`, `trust.rs`, `agent_credentials_routes.rs`, `main.rs`, `crates/storage-sqlite/src/invocations.rs`, `crates/storage-sqlite/src/trust.rs`, `apps/litecowork-ui/src/discover/DiscoverPage.tsx`, `discover-api.ts`, `discover-state.ts`, `needs-you/ApprovalDetail.tsx`, `needs-you/approval-state.ts`, and the composed `docs/schemas/served-route-inventory.json`.

### Current route truth

- Served read/write operations include Discover search/detail; capability `connect-or-install` and `test`; Approval list/detail/resolve; Task-planning read-grant create/revoke; and CapabilityInvocation task-list/detail GETs.
- The `connect-or-install` route requires authenticated owner Workspace/runtime scope and a bounded idempotency key, but it always returns `503 DEPENDENCY_UNAVAILABLE` with reason `LITESPM_PACKAGE_SERVICE_NOT_QUALIFIED`. It creates no durable package/install receipt and explicitly reports `requested_scope_granted=false`.
- The `test` route is a no-provider probe. It returns `health=UNKNOWN` and a reason saying the LiteSPM package service is not qualified and no provider was contacted.
- Connection list/get/disconnect operations in the OpenAPI contract are target-only: `GET /connections`, `GET /connections/{connectionId}`, `POST /connections/{connectionId}/disconnect`. No corresponding composed daemon routes exist. No account-Connection OAuth start/callback/refresh lifecycle is served.
- CapabilityInvocation Operator exposes only GET list/detail. Its module-level comment says it does not create, transition or dispatch Invocations. The SQLite transition store explicitly rejects transition to DISPATCHED with `INVOCATION_DISPATCH_ADMISSION_UNAVAILABLE`. No production invocation-creation/dispatch caller was found in `apps/litecoworkd/src`.
- The Trust evaluator module is compiled with `allow(dead_code)` in `main.rs`; `trust.rs` itself says it does not dispatch providers, issue grants/SecretLeases or create reusable permits, and that CapabilityBroker admission remains fail-closed until the combined Trust/Effect/audit transaction is implemented.
- `OsSecretStore` is a real local OS keychain adapter and the SecretStore API is write-only, but no Feature 006 account Connection service uses it. The mounted AgentProfile credential-slot route is Feature 002 and currently fails 503 after descriptor validation, before SecretStore write, because exact durable replay protection is not ready.

### Approval, grants and UI

- Approval list/get/resolve are served. Approve requires the current Approval version, an idempotency key, local authenticated owner assurance and an exact `confirmation_digest`; the store commits `approval.resolved.v1` with replay protection. Deny does not need the approval digest.
- `ApprovalDetail` has an actual load/resolve UI. It preserves the same decision request identity after an uncertain response, refreshes status, and says explicitly that approving records the exact-action decision only—it does not grant reusable capability access or run the external action.
- The served CapabilityGrant route currently creates a bounded, expiring Task-planning `workspace.resource.read` grant over explicit Resource refs. It does not establish OAuth/account Connection custody or a connector write grant.
- Discover has Explore, Installed/native, Managed and Connections tabs. Explore identifies Workspace-local skills and states public package discovery unavailable. Installed/native shows only Runtime-reported capability hosts (not all native installations). Managed says package management is not connected. Connections says account connections cannot be read in this build.
- `connectActionOutcome()` returns `allowed=false` for every row: availability-not-ready rows show their truthful state/reason; Workspace-local skill Open/Run is unavailable; package Connect/Use is unavailable until the LiteSPM contract is qualified. All row action buttons are disabled.
- No provider OAuth form/callback, account Connection list/detail, provider read/write operation picker, grant assignment, or connected→granted→invoked UI exists. Approval detail is a separate, real state transition and is not proof that an invocation ran.

These are source/route/UI findings. No LiteCowork account Connection was created, OAuth was not started, no token was stored through a Connection flow, no connector provider was called, no CapabilityInvocation was dispatched and no provider Effect was reconciled.

## Alternatives

- Wait for full multi-provider parity before any slice. Rejected: the release contract asks for one qualified flow per area.
- Generate a complete schema inside this package. Rejected: machine contracts live in `docs/schemas/` and the owner doc.

## Limitations

- Rival web pages were not re-fetched on 2026-11-06 for this package; the LiteSPM bridge pin was resolved 2026-11-09 to HEAD `84d5fab` (v0.6.0, [LITESPM-INTEGRATION-STUDY-2026-11.md](../../docs/LITESPM-INTEGRATION-STUDY-2026-11.md)), superseding the earlier `111fd9b8e84573f8fce76b6374b4ff9639502539` as checked on 2026-11-05, not as proof of LiteCowork integration.
- No LiteCowork runtime or end-to-end bridge integration was executed. The upstream source review is static and commit-pinned; it does not qualify a local LiteCowork process launch, IPC identity, host mutation, or receipt/recovery path.

## Provider documentation check (2026-11-09)

| Source | Finding | Effect on this package |
|---|---|---|
| [GitHub: Authorizing OAuth apps](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps) | PKCE supported with `S256` only; loopback `redirect_uri` on any port at `127.0.0.1` is accepted; `client_secret` is listed as required for the web-flow code exchange and for refresh unless the token came from the device flow; the device flow needs no client secret; token exchange is at `github.com/login/oauth/access_token`. | B-19's loopback-PKCE recipe still needs a client secret, so B-19's stop clause applies until the owner decides (TASKS T02). The OAuth token endpoint must be a separate allowlisted auth host. |
| [GitHub REST: OAuth authorizations](https://docs.github.com/en/rest/apps/oauth-applications) | `DELETE /applications/{client_id}/token` requires Basic auth with the client secret. | Remote revocation is not possible without a client secret; revoke is local first and remote revoke may be an owner action (AC-015). |

Content was rephrased for compliance with licensing restrictions.

## Implementation placement notes

- Keep OAuth, token custody and brokered HTTP in the Rust daemon (`apps/litecoworkd`), not in TypeScript or the Tauri shell, so verifiers and tokens never reach the WebView process. The daemon already has `reqwest` with rustls.
- Reuse `OsSecretStore` and its `keyring` backends; add a crate-private resolver and a delete rather than a second secret store.
- Reuse `SqliteCapabilityDispatchStore::admit_capability_dispatch` rather than writing a second admission path.
- Keep UI state logic in small TypeScript modules (`connections-state.ts`) so it is testable with the existing `node --test` runner.

## Unresolved questions

- OAuth recipe after the client-secret finding (TASKS T02, owner).
- Whether the Task-scoped golden write needs an admitted Environment (SPEC §18; affects the Windows row through 005 B-35).
- Confirmation of the golden read operation (proposed: list issue comments), recorded in T01.
