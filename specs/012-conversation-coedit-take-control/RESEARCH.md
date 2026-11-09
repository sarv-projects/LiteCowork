# Research - Conversation co-edit and take-control

**Feature:** 012 | **Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.
**Checked:** 2026-11-08

## Sources

Authorities: docs/STATE-MACHINES.md (turn/cancellation lifecycle), docs/EXPERIENCE.md and docs/API.md (Conversation UX and routes), docs/EVENTS.md (event contracts), feature 001 (turn/cancel authority), feature 004 (relay boundary, to avoid collapsing co-edit into relay). Research gate RV-14 is owned by [docs/VIABILITY-VS-COMPETITORS.md](../../docs/VIABILITY-VS-COMPETITORS.md). Reject vendor co-edit surfaces that share one hidden transcript or create a hidden second worker.

## Decision note - which List-3 gaps this package owns vs existing owners

[docs/SUPERSET-GAP-MATRIX.md](../../docs/SUPERSET-GAP-MATRIX.md) List 3 enumerates 14 capability gaps. This package creates exactly one new owner, for the single un-owned gap; every other item already has a home that was deepened in the 001-011 packages or is a documented research/deferred decision. No redundant package is created.

| List-3 item | Gap | Owner home (existing vs this package) | Why |
|---|---|---|---|
| 1 | Real integrated terminal | Feature 008 (B-23) | Terminal provider is already owned and deepened in 008; co-edit adds nothing there. |
| 2 | Real browser / computer use | Feature 008 (RV-04, B-22, B-32) | Managed-Chromium browser and computer-use decisions live in 008. |
| 3 | Professional document formats | Feature 007 (B-21, B-30, B-31) | Artifact format engine, create route and export receipt are owned by 007. |
| 4 | Memory + knowledge continuity | Feature 009 (B-24; A-34/A-20) | Coworker scoped memory and the single learning vocabulary are owned by 009. |
| 5 | Qualified native turn + relay | Features 001 and 004 (RV-02/RV-03, B-15/B-16) | Turn authority is 001; A->B->A relay and portable context are 004. |
| 6 | LiteSPM connect/install + invocation | Feature 006 (RV-01, A-13, A-33) | Capability write and connection are owned by 006. |
| 7 | Trust / approvals / isolation / mounts | Feature 005 (RV-05) | Safe Task path, Environment isolation and mount modes are owned by 005. |
| 8 | Scheduled tasks with honest columns | Feature 010 (RV-13) | TriggerHost and occurrence/notification truth columns are owned by 010. |
| 9 | Routines / skills, teach-to-skill | Feature 010 + research RV-06 | Routine-to-Task is owned by 010; teach-to-skill stays research (RV-06). |
| 10 | Event/condition proactivity + Needs You | Feature 010 (B-26) + RV-07 | ON_CONDITION and triage surface are owned by 010/004; no new package. |
| 11 | Notification delivery | Feature 010 (B-27; A-27/A-28) | Local channel and quiet-hours delivery are owned by 010. |
| **12** | **Synchronous human+agent co-edit / take-control** | **This package 012 (B-34, RV-14)** | **The only gap with "Doc claims it? = no" and no owner; mapped onto a Conversation take-control / side-chat UX, no new durable-Task type.** |
| 13 | Screen-understanding / audio-capture triage | No owner yet; consent-gated RESEARCH | Needs a dedicated privacy/permission owner before any target; deliberately not folded into a Conversation UX package. |
| 14 | Voice / realtime audio agent | Research RV-08 (deferred) | Deferred with its own privacy/provider contract; not a co-edit concern. |

Conclusion: only item 12 lacked an owner and is a natural Conversation UX, so it is the sole new package. Items 13 and 14 are intentionally left to future privacy/provider owners rather than forced into this package.

## Findings

- The owner documents already decide the Conversation turn, single-flight lane and cancellation invariants this slice must keep. Co-edit adds take-control and side-chat affordances on top; it does not redefine any of them.
- RV-14 (Claude Duet) has no resolved first-party URL as of 2026-11-08 (claude.com/product/duet and anthropic.com/news/claude-duet returned 404 in the VIABILITY record). Claude Duet and Claude Code Desktop side-chat are vendor UX references only; no code is reused and no Duet capability is asserted as fact.
- Reference desktop repositories are architecture/UX comparisons, not code to vendor. Pins and paths: [REFERENCE-IMPLEMENTATIONS.md](../../docs/REFERENCE-IMPLEMENTATIONS.md), [REFERENCE-REPOS-CORE-ARCHITECTURE-INDEX.md](../../docs/REFERENCE-REPOS-CORE-ARCHITECTURE-INDEX.md).

## Borrowable-design sources (pinned)

| Source | Exact pinned URL | License | What informs co-edit / take-control | Verdict |
|---|---|---|---|---|
| OpenLoomi "Attention Agent" lifecycle/close-behavior Rust modules (`melandlabs/openloomi`) | https://github.com/melandlabs/openloomi/tree/2aca1016fa3065e2fee912afd0e94576d0483c66 | Apache-2.0 at checked root LICENSE | Same Tauri+Rust stack as LiteCowork; its presence/attention surface and `apps/web/src-tauri` lifecycle modules inform the truthful driver indicator (agent running vs owner in control) derived from durable state, not a live handle. | PROTOTYPE the driver-indicator shape against STATE-MACHINES; do not vendor the modules or adopt a second worker model. |
| OpenDot local notification levels (`athulsreekumar/opendot`) | https://github.com/athulsreekumar/opendot/tree/ec75891b819713acf29cb659bd9c72d1ed34f76a | MIT at checked root LICENSE | Its urgent/update/quiet local attention levels inform how a take-control / side-chat state is surfaced without implying hidden native memory. | PROTOTYPE the attention-level presentation only; keep LiteCowork Conversation identity and turn settlement authority. |
| Claude Duet co-edit (vendor) | https://www.anthropic.com (first-party Duet URL unresolved 2026-11-08; RV-14) | vendor (reference only) | Vendor UX reference for synchronous human+Claude co-edit; it is the market analogue this package maps onto a Conversation UX. | REFERENCE-ONLY, NO CODE. Do not assert as a shipped fact; cite RV-14 as RESEARCH. |
| Claude Code Desktop side-chat (vendor) | https://www.anthropic.com (Code tab side-chat; COMPETITIVE-RESEARCH §Claude Code Desktop) | vendor (reference only) | Partial side-chat analogue that informs the correlated `OWNER_SIDE_NOTE` affordance without steering the running turn. | REFERENCE-ONLY, NO CODE. UX reference, not an owner concept. |

No OpenLoomi, OpenDot or vendor code is imported; all are read at the pinned SHAs or cited as reference-only UX.

## Source and route audit (2026-11-08)

- The composed Conversation route in `apps/litecoworkd/src/conversation_operator.rs` exposes submit/cancel/retry; the cancel route commits `CANCEL_REQUESTED` then `CANCELLED` without observing native interrupt/quiescence, and the desktop Stop handler searches transient turns the OpenCode path never emits (register A-25). There is no take-control continuation or side-chat message kind.
- `AgentSessionSupervisor` dispatches only `provider_key == "opencode"`; take-control reuses its cancel/quiescence authority rather than adding a provider path.
- The `ConversationContextPackage` builder orders committed messages by aggregate revision and excludes `SYSTEM_NOTICE`; it has no side-note concept because no side-note exists yet. The accepted (not yet implemented) exclusion of `OWNER_SIDE_NOTE` from the running turn's prompt is a new requirement, not current behavior.
- Changing co-edit state must not create a Task, Attempt, DelegationProfile, Environment, lease or grant; this matches the Conversation/Task separation enforced elsewhere in the source.

These are source/route/UI findings. No co-edit, take-control or side-chat journey was executed. Feature 012 is ACCEPTED/LOCKED (design only) since 2026-11-09.

## Alternatives

- Model co-edit as a new durable-Task type. Rejected: it collides with the Conversation/Task separation (PRODUCT, DATA-MODEL) and the gap-matrix guidance to map co-edit onto a Conversation UX.
- Feed side-chat text into the running turn's prompt. Rejected: it would let untrusted owner text steer the agent and break the context-package boundary.
- Build a CRDT/operational-transform document co-editor. Rejected: out of scope; co-edit here is a Conversation presentation and take-control UX.

## Limitations

- RV-14 Claude Duet first-party URL unresolved on 2026-11-08; use [COMPETITIVE-RESEARCH.md](../../docs/COMPETITIVE-RESEARCH.md) before any market-positioning decision.
- Take-control cannot be qualified until the Feature 001 Stop/quiescence defects (A-25) are fixed.

## Unresolved questions

See SPEC section 5 blocker B-34 and research gate RV-14. Owners are named on the package README.
