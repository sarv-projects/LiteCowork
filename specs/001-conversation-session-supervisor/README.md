# 001 - Conversation Session Supervisor and Native Turn Execution

- **Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.
- **Revision:** 1.1 | **Updated:** 2026-11-09 | **Owner acceptance:** B-01 to B-10 accepted 2026-11-09 via [docs/OWNER-ACCEPTANCE-REVIEW-PACKET.md](../../docs/OWNER-ACCEPTANCE-REVIEW-PACKET.md); first route to qualify is OpenCode v2.0.32 on Windows x64 (B-02 rider).
- **Implementation state (2026-11-10):** CODE and SERVED, no live agent run and no QUALIFIED row. DONE: T02, T06, T07, T08, T10, T11, T13, T15, T16, T17, T18, T19, T20, T24, T26. PARTIAL: T01, T03, T04, T05 (B-07 window; the real OpenCode input budget is unconfirmed, O-05), T14 (text attachments only), T21, T22, T23, T25 (REPORTED activity), T27 (permission display only). BLOCKED: T12 (no evidenced OpenCode input-request event). NOT STARTED: T09, T28 (need a live OpenCode run and release evidence). Migrations v28 to v30 are wired. Details: TASKS.md and docs/IMPLEMENTATION-STATUS.md.
- **Owners:** Conversation, Agent Runtime, Storage, Operator/API, Desktop UI, Security/Trust and Product.

## User outcome

A user can send an ordinary LiteCowork Conversation message through the exact selected native agent, see truthful progress, stop/retry safely and reload the durable answer after restart, without creating a Task or Coworker.

## Current truth

The active source contains an OpenCode-only supervisor path that reserves and activates a Conversation AgentSession, loads/sends bounded context, consumes the OpenCode response stream and commits the final AGENT message with terminal turn/session settlement. This is CODE-level integration evidence, not a qualified Windows desktop Send journey. Since commit 8129420 the mounted UI gates Send behind the `sendEligible` predicate (register A-24, a UI gate only, not G-09 qualification) and the empty state is truthful and LiteCowork-branded (register A-26 empty-state part); both are guarded by `apps/litecowork-ui/tests/conversation-empty-state-truthfulness.test.mjs`. Stop is still ineffective for the OpenCode path, Retry has no UI call site, and composer attachments/voice/more-options remain inert. The commit-to-spawn crash window has no durable dispatch outbox/recovery sweep; some supervisor errors are only logged; retry transitions to RUNNING before a reservation path that requires OPEN; and the storage submit path does not enforce the accepted single-flight policy. Keep Send/release claims gated until those issues and exact provider/OS/UI qualification are resolved.

By owner approval on 2026-11-09 (see [docs/OWNER-ACCEPTANCE-REVIEW-PACKET.md](../../docs/OWNER-ACCEPTANCE-REVIEW-PACKET.md)), this package also folds the audit's cowork-visibility tasks P-00, P-01, P-02, P-04 and P-09 (001-T23 to 001-T27): a redacted OpenCode v2.0.32 turn fixture, the OpenCodeEventNormalizer, transient activity frames with REPORTED records, stick-to-bottom autoscroll, and native permission prompts surfaced inline and in Needs You (jointly with Feature 002). Source: [docs/UI-RENDERING-SETTINGS-ARCHITECTURE-AUDIT-2026-11.md](../../docs/UI-RENDERING-SETTINGS-ARCHITECTURE-AUDIT-2026-11.md) section 9.

## Package

- [SPEC.md](SPEC.md) - target HLD/LLD, decisions, security and EARS acceptance.
- [PLAN.md](PLAN.md) - phases, dependency DAG, risks and gates.
- [TASKS.md](TASKS.md) - reviewable implementation tasks and blockers.
- [DATA-MODEL.md](DATA-MODEL.md) - reused records, implemented result-write boundary and remaining durability gaps.
- [RESEARCH.md](RESEARCH.md) - sources, verified evidence, alternatives and open questions.
- [WIRING.yaml](WIRING.yaml) - component/owner/transaction graph.
- [contracts/](contracts/README.md) - API/event and failure/recovery contract use.

## Canonical owners

[Architecture](../../ARCHITECTURE.md), [Product](../../docs/PRODUCT.md), [V1 release](../../docs/V1-PRODUCT-RELEASE-CONTRACT.md), [Agent Control](../../docs/AGENT-CONTROL.md), [Agent Fabric](../../docs/AGENT-FABRIC.md), [Native Composer LLD](../../docs/NATIVE-COMPOSER-LLD.md), [State Machines](../../docs/STATE-MACHINES.md), [Services](../../docs/SERVICES.md), [Storage](../../docs/STORAGE.md), [Events](../../docs/EVENTS.md), [API](../../docs/API.md), [Schemas](../../docs/SCHEMAS.md), [Trust](../../docs/TRUST.md), [Failure Recovery](../../docs/FAILURE-RECOVERY.md), [Testing](../../docs/TESTING.md).

