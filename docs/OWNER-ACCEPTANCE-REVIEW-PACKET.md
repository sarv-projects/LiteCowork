# Owner acceptance review packet

**Status:** decision request to the named owners. This is NOT an acceptance record and NOT an authority doc; it collects every PROPOSED resolution in one place so owners can approve/reject in a single pass. Per [SPECIFICATION-STANDARD.md](SPECIFICATION-STANDARD.md) §7 an author may not self-approve; nothing here is accepted until an owner records a decision.
**Assembled:** 2026-11-08, after the doc audit, the doc+spec upgrade pass and the A-24/A-26 code fixes.
**How to use:** for each row write **ACCEPT**, **REJECT** (with the alternative), or **DEFER** in the Decision column, then hand this back. On ACCEPT, the owning package can advance DRAFT → IN_REVIEW → ACCEPTED/LOCKED and implementation is authorized for that package. Each resolution's full text lives at its owner (register row or specs/NNN §5); this packet summarizes and links, it does not restate authority.

Legend for my recommendation: **ACCEPT** = adopt as written; **ACCEPT (scoped)** = adopt but note a caveat; **NEEDS YOU** = a genuine product/strategy call I should not pre-bias.

---

## Part 1 — Package 001 (Conversation Session Supervisor) — the implementable one

Package 001 is `IN_REVIEW` and is the only package deep enough to implement immediately. Accepting B-01..B-10 moves it to ACCEPTED/LOCKED and unblocks tasks 001-T01..T09. Full text: [specs/001-.../SPEC.md](../specs/001-conversation-session-supervisor/SPEC.md) §5.

| ID | Decision needed | Proposed resolution (summary) | My rec | Trade-off / why | Decision |
|---|---|---|---|---|---|
| B-01 | Concurrent-send policy per Conversation | Strict single-flight turn lane; 2nd submit → `CONVERSATION_TURN_IN_PROGRESS` 409; composer disabled while active | **ACCEPT** | Simplest correct model; eliminates HLC-ordering/correlation races. Cost: no parallel turns in one Conversation (fine for chat). | |
| B-02 | First GA OS + harness + adapter + auth | Windows 11/10 `x86_64-pc-windows-msvc` + Claude Code & OpenCode/Codex CLI; Adapter 1.0 framed subprocess; harness-managed native pass-through auth | **NEEDS YOU** | This is the single biggest strategic call — it sets the first qualified platform and harness. Windows-first matches the existing test history; but the only path that actually *runs today* is OpenCode (the audit found Codex/Claude Code not wired to the supervisor). Consider **ACCEPT but name OpenCode as the first route to qualify** so the spec matches reality, with Claude Code/Codex as fast-follows. | |
| B-03 | Readiness handshake proving session identity | Three-phase: version probe → create-session → atomic SQLite CAS activation; prompt dispatched only after activation commits | **ACCEPT** | Cleanly separates host-process presence from turn readiness (a core invariant). No real downside. | |
| B-04 | Atomic final-response commit | `commit_conversation_agent_response` IMMEDIATE txn: message + turn SETTLED + session CLOSED + 2 events + dedup receipt, all-or-nothing | **ACCEPT (scoped)** | Correct shape and already partly coded. Caveat the audit found: add explicit **retry-ordinal + stable provider-response identity** fields (currently missing) before calling it complete — that's task 001-T02/T06. | |
| B-05 | Streaming resume vs gaps | Best-effort transient `TURN_DELTA` frames; on gap, discard buffer and REST-resync the committed transcript; never replay transient frames | **ACCEPT** | Keeps durable truth in storage, stream as projection (invariant-aligned). | |
| B-06 | Conversation-scoped UserRequest continuation | Commit immutable response → WAITING_USER → WAITING_DEPENDENCY; reconcile provider input key; fresh session only after acceptance; never creates Task/Attempt | **ACCEPT** | Preserves Conversation≠Task separation. | |
| B-07 | Context reduction for tight budgets | Pinned preamble + selected refs + recent contiguous messages to 80% of budget; explicit truncation receipt in prompt | **ACCEPT (scoped)** | Reasonable deterministic window. Note open question O-05: a pinned harness may accept less than the 4 MiB/1,024-message package; 256 KiB in API.md is a different cap. Confirm the first adapter's real budget during 001-T05. | |
| B-08 | Attachment/resource materialization | Text inline with checksummed headers; binary as verified read-only WorkspaceRoot paths via `--file`; reject arbitrary external paths | **ACCEPT** | Enforces Workspace boundary / prevents path traversal. | |
| B-09 | Process ownership + idle shutdown | Job Object (Win)/cgroup (Linux); 10-min idle TTL; graceful term → 5s → kill; never kill shared external processes | **ACCEPT (scoped)** | Good default. Caveat per invariant: a Job Object is **not** a full filesystem/network/credential sandbox — don't let this row imply isolation (that's B-17/RV-05). | |
| B-10 | Usage/cost attribution | Provider-reported tokens if present, else byte heuristic (`bytes/4`) marked `ESTIMATED_LOCAL`; never block settlement on missing telemetry | **ACCEPT** | Truthful, non-fabricated accounting; display never fabricates $0.00/0 tokens. | |

**001 open conformance items (not blockers, but gate VERIFIED):** O-01 first route must match source (only OpenCode runs today), O-03 single-flight not yet enforced in storage, O-04 final-txn provenance/rollback tests incomplete. These are implementation tasks, not acceptance decisions.

---

## Part 2 — Cross-cutting owner decisions (register A-xx)

Full text: [CONTRADICTION-AND-ALIGNMENT-REGISTER.md](CONTRADICTION-AND-ALIGNMENT-REGISTER.md).

| ID | Decision needed | Proposed resolution (summary) | My rec | Trade-off | Decision |
|---|---|---|---|---|---|
| A-18 | ContextDocument create kinds | Narrow OpenAPI *create* enum to the 2 validator-accepted kinds (PERSONAL_PROFILE, WORKSPACE_NOTES); keep all 4 in record/read/policy as TARGET-pending | **ACCEPT** | Makes the machine contract match the validator today; the 2 deferred kinds stay target values. No downside. | |
| A-20 / A-34 | The single learning vocabulary | Choose `LearningCandidate` as the one candidate-lifecycle vocabulary (owner LEARNING.md); keep `MemoryRecordMetadata.status` as the distinct committed-record lifecycle; supersede `MemoryCandidate` | **ACCEPT** | Ends a 3-way vocabulary collision with one owner. `LearningCandidate` is the richer/more-expressive set. Unblocks 009/B-24. | |
| A-35 | Settings screen ownership | Settings→Agents **screen** owned by Feature 011 (002 links); quiet-hours control owned by Feature 010 (011 links) | **ACCEPT** | One owner per screen. Matches where each capability actually lives. Unblocks 002/010/011 from leaving DRAFT. | |
| A-37 | delegation.schema `on_exhaustion` | Keep the schema (it matches accepted DELEGATION target); mark the field TARGET-not-yet-wired; packets omitting it are valid until the dispatcher exists | **ACCEPT** | Schema-ahead-of-code by design; honest TARGET label. Wiring is a 004/005 step. | |

---

## Part 3 — Packages 002–012 blockers (each gates its package leaving DRAFT)

Each resolution's full text is in the named `specs/NNN §5`. Accept to let that package advance.

| ID | Pkg | Decision | Proposed resolution (summary) | My rec | Trade-off | Decision |
|---|---|---|---|---|---|---|
| B-11 | 002 | Additional AgentModules after first route | Qualify exactly one more module matching the B-02 pair | **ACCEPT (scoped)** | Depends on the B-02 call; keep in lockstep. | |
| B-12 | 002 | Descriptor freshness TTL / probe budget | 15-min per-module TTL, 3/min probe budget, module-overridable | **ACCEPT** | Sensible defaults; overridable. | |
| B-13 | 003 | First ingestion formats | Qualify the allowlisted UTF-8 lexical text set first | **ACCEPT** | Smallest safe surface; formats added later. | |
| B-14 | 003 | Watcher vs explicit rescan | Manual rescan is GA default; no V1 file watcher | **ACCEPT** | Avoids OS-notification complexity/races in V1. Cost: no live auto-refresh (acceptable). | |
| B-15 | 004 | Portable context size/reduction for relay | Reuse 001 package bounds + portability filter + audited truncation | **ACCEPT** | No private-transcript migration (invariant-safe). | |
| B-16 | 004 | Second qualified route + relay semantics | Second module = the B-02 pair complement; fresh session per turn | **ACCEPT (scoped)** | Tied to B-02. | |
| B-17 | 005 | First Environment provider | Linux **bubblewrap** with all four isolation dimensions ENFORCED | **NEEDS YOU** | Strategy: this picks the first real sandbox. Bubblewrap (Linux) is a solid first target and contradicts nothing, but it means the first *Task-execution* platform is Linux while the first *chat* platform (B-02) is Windows. Confirm you're comfortable with that split, or ask for a Windows-first Environment instead. | |
| B-18 | 005 | First mediated file-op class | Atomic whole-file replace, ≤1 MiB, confined path, temp-rename, verifier | **ACCEPT** | Smallest provable Effect with rollback. | |
| B-19 | 006 | First Connection/provider | GitHub public PKCE loopback client; `api.github.com` allowlist | **ACCEPT** | No embedded secret (PKCE); well-understood API. | |
| B-20 | 006 | First capability write | Create an issue comment (WRITE class); UNKNOWN blocks + reconciles | **ACCEPT** | Low-blast-radius first write with idempotency. | |
| B-21 | 007 | First professional Artifact format | **Markdown** `text/markdown`, deterministic in-process serializer, byte-stable round-trip; **no Office engine** in first slice | **ACCEPT** | Pragmatic: proves the Artifact pipeline without an Office/PDF engine. Cost: the headline PPTX/DOCX/XLSX parity (gap-matrix #3) is deferred — intentional. | |
| B-22 | 008 | First real browser mode | `MANAGED_CHROMIUM` isolated profile; ambient-CDP discovery & default-profile copy DISABLED; attach-existing later | **ACCEPT** | Security-first (the browser-use adapter rule). Cost: no "use my logged-in Chrome" in first slice. | |
| B-23 | 008 | First real terminal provider | One daemon-owned PTY/ConPTY per tab under an admitted Environment; containment receipt; ordered receipts; timeout-as-receipt; no startup-script exec during discovery | **ACCEPT** | Directly fixes the fabricated-terminal defect; borrows OpenHands receipt doctrine. | |
| B-24 | 009 | Minimal Coworker memory record | Owner-authored Coworker-scoped ContextDocument using `MemoryRecordMetadata.status`; automatic `LearningCandidate` pipeline deferred; validator accepts `COWORKER_NOTES` | **ACCEPT** | Depends on A-34 ACCEPT. Ships manual memory first, auto-learning later. | |
| B-25 | 009 | Quick Create Coworker binding | Starts UNBOUND until Feature 002; Coworker + first-chat are separate idempotent commits | **ACCEPT** | Clean separation; safe retry. | |
| B-26 | 010 | First Automation trigger type | Cron-in-named-timezone first; `RUN_ONCE_WHEN_AVAILABLE` default; stored-TZ DST; bounded catch-up | **ACCEPT** | Standard, honest schedule semantics. | |
| B-27 | 010 | Notification channel/guarantees | Single LOCAL_POLL channel; `PENDING` delivery row distinct from Task outcome; quiet-hours suppression; no webhook; no delivery guarantee | **ACCEPT** | Fixes the "notifications inserted but never sent" defect honestly. | |
| B-28 | 011 | First GA OS + security profile | Windows 11/10 `x86_64-pc-windows-msvc` first (named-pipe DACL); Linux 0600/0700 socket second; WSL2 never counts | **NEEDS YOU** | Must agree with B-02. If you pick Windows-first for chat, accept this as-is. | |
| B-29 | 011 | Which Settings groups enabled in V1 | Only served-owner groups interactive (Workspaces, Runtime reads, Agents via 002); others visibly unavailable | **ACCEPT** | Honest; no fake settings. | |
| B-30 | 007 | Artifact create authority | One `POST /v1/artifacts` create, 3 source classes, one IMMEDIATE txn | **ACCEPT** | Needed for 007; atomic. | |
| B-31 | 007 | Durable export receipt | `ArtifactExportReceipt` + `artifact.exported.v1` distinct from native Save As; destination class not raw path | **ACCEPT** | Distinguishes real export from local save. | |
| B-32 | 008 | Computer-use milestone | **Defer** from first V1 milestone; keep surface explicitly UNAVAILABLE (no disabled-but-ready control) | **ACCEPT** | Scopes V1 down sensibly; computer-use is the riskiest surface. | |
| B-33 | 011 | V1 Settings acceptance surface | The honestly-mapped served controls within the mounted 4 categories are the acceptance surface; the 16-category IA is authored target, not an assertion | **ACCEPT** | Resolves a 2-orders-of-magnitude mismatch between contract and shipped UI toward honesty. | |
| B-34 | 012 | Co-edit / take-control owner model | Two bounded Conversation affordances over the 001 turn authority (take-control via cancel/quiescence + `OWNER_TOOK_CONTROL` event; `OWNER_SIDE_NOTE` side-chat excluded from prompt); no new Task type | **ACCEPT (scoped)** | Depends on the 001 Stop/quiescence fix (A-25) being real before it can work; cites RV-14 as research, not fact. Accept the *model*; implementation waits on 001. | |

---

## Summary for the owner

- **To start building immediately:** accept **Part 1 (B-01..B-10)** → package 001 goes ACCEPTED/LOCKED and tasks 001-T01..T09 are authorized. The one real strategic call there is **B-02** (first OS/harness); my recommendation is to accept it *but name OpenCode as the first route to qualify* so the spec matches what runs today.
- **To unblock the next tier** (install/auth, roots, safe-task): accept **A-35, A-18, A-34/A-20** (cross-cutting) plus **B-11..B-20**.
- **Genuine product calls I did not pre-bias (NEEDS YOU):** **B-02 / B-28** (first GA OS — Windows-first vs OpenCode-reality) and **B-17** (first Environment sandbox — Linux bubblewrap, which splits the first chat platform from the first Task platform).
- Everything else I recommend **ACCEPT** (or ACCEPT-scoped with the noted caveat).

No decision here is recorded as accepted. Return this with a Decision per row and I will mark the accepted packages, advance their lifecycle status, and begin implementation of the first accepted package.
