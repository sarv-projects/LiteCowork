# Audit: Bare-minimum release = superset of competitor **maximum**

**Date:** 2026-11-08  
**Release bar (product owner):** LiteCowork’s **floor** for shipping is **not** “one qualified slice per pillar.” It is: **at ship date, every capability competitors offer at their documented maximum must work in LiteCowork, and LiteCowork may add more** (durable Task, trust/effects, LiteSPM grants, evidence, Coworkers, recovery).  
**This audit grades only against that bar.** It does **not** use `V1-PRODUCT-RELEASE-CONTRACT.md` “one flow per area” as the pass line.

**Understanding:** Yes — **minimum release ⊆ competitor-max envelope, with strict superset intent.**

---

## 1. Competitor-max envelope (ship-date baseline)

We define **competitor max** as the **union** of what leaders ship **today** on their **strongest qualified surface** (desktop or IDE where that is their peak), from primary docs/repos checked 2026-11-08:

| Source | “Max” surface (what users can actually use) | Primary evidence |
|---|---|---|
| **Claude Code desktop** | Multi-session sidebar, integrated **terminal**, **diff**, **browser preview**, computer use, side chats, connectors, scheduled/local tasks, plugins parity with CLI | [code.claude.com/docs/en/desktop](https://code.claude.com/docs/en/desktop/) |
| **OpenAI Codex CLI + app integration** | Repo edit loop, **sandbox tiers**, `codex exec` CI, resume/fork, MCP, skills/plugins, code review mode, worktrees, permissions | [learn.chatgpt.com/docs/codex/cli](https://learn.chatgpt.com/docs/codex/cli.md) |
| **OpenWork** | Desktop on Win/Mac/Linux, **OpenCode** harness, 50+ providers / Ollama, skills+MCP, browser automation claims, schedules, local files, team MCP gateway | [github.com/different-ai/openwork](https://github.com/different-ai/openwork) |
| **Open Cowork (OpenCoworkAI)** | One-click agents, **WSL2/Lima VM** sandbox, MCP, **real terminal**, GUI/computer use, PPTX/DOCX/XLSX/PDF skills, Feishu/Slack remote | [github.com/opencoworkai/open-cowork](https://github.com/opencoworkai/open-cowork) |
| **Cline** (IDE max) | **Real terminal**, headless browser tests, MCP marketplace, checkpoints/undo, plan/act, skills/hooks, multi-IDE | [docs.cline.bot](https://docs.cline.bot/cline-overview), [cline.bot/ide](https://cline.bot/ide) |
| **Claude Cowork / ChatGPT Work** (knowledge-work max) | Persistent assistant, **schedules** (some cloud continuation), approvals, local folder work via desktop bridge | Anthropic/OpenAI help articles in `docs/COMPETITIVE-RESEARCH.md` |

**LiteCowork extras (must be present *in addition*, not instead of):** multi-harness BYOA without replacing native behavior; durable Task/Attempt/Effect; Trust on mediated ops; LiteSPM lifecycle; Coworker scoped memory (when claimed); Workbench takeover/leases; evidence (reported/observed/verified).

---

## 2. Pass criterion per capability row

For each row below:

- **PASS (superset-min)** = Qualified USER journey on declared OS; competitor-max behavior matched or exceeded; honest UX if intentionally stronger boundary.
- **FAIL** = Absent, stub, 503, UI-only, code without USER qualification, or **weaker** than competitor max.

---

## 3. Superset-minimum matrix

| # | Capability (competitor max union) | Competitor max behavior | LiteCowork now | vs max | Status |
|---|---|---|---|---|---|
| **A** | **Ordinary chat, no setup tax** | Message → stream → stop; history; search | Create/list/send path; OpenCode code-only; durability/cancel gaps | **Weaker** | **FAIL** |
| **B** | **Multi-model / BYOA** | Install/sign-in/configure **multiple** harnesses; native controls | Catalog + probes; **one** dispatch branch; OpenCode bind blocked; install/auth 503 | **Much weaker** | **FAIL** |
| **C** | **Real terminal** | PTY/ConPTY/shell; streaming; user sees truth | `TerminalPane` = **unavailable** (honest) | **Absent** | **FAIL** |
| **D** | **Real diffs / review** | VCS or editor diffs; accept/reject | Review pane **unavailable** | **Absent** | **FAIL** |
| **E** | **Agent-driven browser** | CDP/headless; agent navigates & tests | No browser provider; policy code `dead_code` | **Absent** | **FAIL** |
| **F** | **Computer / GUI use** | Screen + input where offered | `environment_computer_use` refuses all | **Absent** | **FAIL** |
| **G** | **Sandboxed execution** | Codex sandbox / Open Cowork VM / job containment **used** | Types + Job Object hints; **no** qualified attempt path | **Absent** | **FAIL** |
| **H** | **MCP / skills / plugins at max** | Install, run, extend agent | Discover read; LiteSPM **503**; no qualified install/adopt | **Much weaker** | **FAIL** |
| **I** | **Repo/code editing loop** | Multi-file edit + run tests + fix | Resources + text preview; **no** agent file loop via Workbench | **Weaker** | **FAIL** |
| **J** | **Checkpoints / undo / resume** | Cline checkpoints; Codex resume/fork | Conversation versions partial; resume not qualified | **Weaker** | **FAIL** |
| **K** | **Parallel sessions** | Claude Code sidebar sessions | Multi-conversation list yes; **not** parallel qualified agents | **Weaker** | **FAIL** |
| **L** | **Scheduling / automations** | Cron-like; runs **agent work** | Host materializes **READY Task**; **no** execution; notifications unsent | **Weaker** | **FAIL** |
| **L2** | **Laptop-off / cloud continue** | Claude/Cowork cloud (competitor max) | V2 target; local V1 | N/A if V1 local-only | **FAIL** if owner includes in min bar |
| **M** | **Approvals + consequential tools** | Human-in-loop; then **runs** | Approvals resolve; **no** wired effect execution | **Weaker** | **FAIL** |
| **N** | **Office / PDF artifacts** | Open Cowork skills; real edit/export | Plain text append; Resources for Office/PDF | **Much weaker** | **FAIL** |
| **N2** | **Connectors (Slack/GitHub/etc.)** | Claude connectors; MCP | Connections **target-only** | **Absent** | **FAIL** |
| **O** | **Persistent coworker persona** | Cowork/Dots-style memory + chat | Coworker **config** only; **no** owned chats/memory | **Weaker** | **FAIL** |
| **P** | **Headless / CI (`codex exec`)** | Scriptable agent | CLI exists; **not** productized as competitor CI surface | **Weaker** | **FAIL** |
| **Q** | **Settings / polish at max** | Full prefs, shortcuts work | **4** settings categories; theme ephemeral | **Weaker** | **FAIL** |
| **R** | **LiteCowork-only extras** | — | Task/Evidence **schema**; Trust **unwired**; LiteSPM **unwired** | **Extras also FAIL** | **FAIL** |

### Summary counts (superset bar)

| | Count |
|---|---|
| Rows **PASS** | **0 / 19** (0%) |
| Rows **FAIL** | **19 / 19** |
| Rows where **extras alone** could differentiate | **0** qualified |

**Verdict on your bar:** The repository **does not** meet “bare minimum = their all max, superset.” It is not close on Workbench providers, BYOA breadth, MCP lifecycle, Task execution, or qualified chat.

---

## 4. What *is* done (foundation only — **does not** satisfy superset min)

These are **real** but are **table stakes engineering**, not competitor-max product:

- SQLite domain model + migrations + hundreds of **unit** tests (`storage-sqlite`, Windows workspace cited in `IMPLEMENTATION-STATUS.md`).
- Composed Operator **173 served** routes (many read-only / fail-closed).
- Desktop UI shell, conversations UI, Work/Automations/Library/Discover/Settings surfaces.
- OpenCode supervisor **code path** (not USER-qualified; binding probe incompatible).
- Resource upload, lexical retrieval API, approval **decision** persistence.
- Automation **trigger host** (schedules → READY Task only).
- Workbench **honest** unavailable terminal/review (better than fake demos, still **FAIL** vs max).

---

## 5. What must be true to ship on **your** bar (ordered)

**Rule:** No row in §3 may remain FAIL. Competitor max is re-baselined at ship date (new features add rows).

### Phase 0 — Non-negotiable parity block (competitor max core)

1. **Qualified multi-harness chat:** Codex + OpenCode + at least one of Claude Code / Gemini CLI paths — install, auth, native controls, stream/stop/resume, attachments, **same** as each harness’s max on that OS.
2. **Real terminal service** (daemon PTY, environment, streaming, kill/cleanup) — **Cline/Open Cowork class**.
3. **Real diff/review** from agent edits (git or revision pairs) — **Claude Code desktop class**.
4. **Managed browser session** (CDP or qualified WebView controller) — **Cline + Claude Code class**, not iframe URL toy.
5. **Sandboxed command execution** with enforced roots — **Codex sandbox + Open Cowork VM** (platform matrix declared).
6. **LiteSPM full lifecycle** — discover, preview, install/adopt/update/repair/remove with receipts — **OpenWork class**.
7. **MCP/skills usable end-to-end** — install server, grant, invoke, evidence — **Cline/OpenWork class**.
8. **Coding loop** — agent edits files, runs tests in terminal, user reviews diff — **competitor max union**.
9. **Schedules that run agent work** — not only READY Task rows — **Cowork/Work schedules class** (local awake semantics explicit).
10. **Connectors OAuth + one write path** with reconciliation — **golden connection** qualified.
11. **Office/PDF or explicit parity doc** — if Open Cowork max includes DOCX/PPTX, ship qualified engine **or** amend bar (owner decision).

### Phase 1 — Superset (why LiteCowork exists)

12. Task execution vertical: plan → attempt → effect → **verified** outcome, crash-safe.
13. Trust wired at **effect boundary**, not UI copy.
14. Coworker: owned conversations + scoped memory with review inbox.
15. Evidence UI: reported / observed / verified everywhere competitor shows only logs.

### Phase 2 — Ship gate

16. **Superset checklist** automated: each competitor-max row has USER test ID on Windows + Linux + macOS per declared matrix.
17. Re-baseline competitors **30 days before RC**; new max rows → new FAIL until qualified.

---

## 6. Contradiction with repo authority (must fix to avoid building wrong thing)

| Document | Says | Your bar |
|---|---|---|
| `V1-PRODUCT-RELEASE-CONTRACT.md` §3 | “One qualified real flow **in every area**” | **Every competitor max capability in every area** |
| `IMPLEMENTATION-STATUS.md` | Pre-release; lists honest gaps | Aligns with **FAIL** on superset, not with ship |
| `specs/README.md` | Features 002–011 **DRAFT**; 001 IN_REVIEW | Insufficient for superset-min |
| Release blockers list (12 items) | Smaller than §5 here | **Underscoped** for your bar |

**Proposed canonical decision:** Amend product contract: **“V1 minimum release = competitor-max envelope (§1) + LiteCowork superset (Phase 1), all rows PASS.”** Until amended, engineering will keep optimizing to the smaller contract.

---

## 7. Effort reality (no dates without staffing)

- **Competitor max union** is **strictly larger** than current V1 contract and **larger** than current codebase by **multiple vertical integrations**, not UI pages.
- **Conservative staffing:** 15–25 senior engineers (runtime, harness, security, UI, QA qualification), **multi-year** if competitor max keeps moving, unless scope is **frozen** to a dated envelope snapshot.
- **Current checkout** is **documentation-heavy prototype** on storage/routes; **0%** superset-min rows PASS.

---

## 8. Answer in one line

**Yes, I understand:** ship means **≥ max(Claude Code desktop, Codex, OpenWork, Open Cowork, Cline IDE, …) plus your differentiators** — **not** “better architecture docs than OpenWork.”

**Audit result:** **You are not building to that bar yet.** The repo, contracts, and implementation status describe a **smaller** release. Changing the slogan without changing the contract and gates will not produce a superset release.

---

*Evidence:* `docs/IMPLEMENTATION-STATUS.md`, `apps/litecowork-ui/src/workbench/WorkbenchPanel.tsx`, `apps/litecoworkd/src/conversation_operator.rs`, `docs/schemas/served-route-inventory.json`, competitor URLs in §1, prior `audit-output/principal-audit-2026-11-08/PRINCIPAL-AUDIT.md`.
