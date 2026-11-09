# LiteCowork

**One workspace for the AI agents you already use.**

LiteCowork is a local-first, chat-first desktop AI workspace built around the native agent harnesses you already run. You chat, delegate durable work, keep persistent Coworkers, and collect the files and evidence of what was done, all in one desktop app, without replacing the agent's own reasoning engine. Bring your own agent; LiteCowork gives it a workspace.

## What LiteCowork does

LiteCowork brings the Final Product Feature List into one coherent desktop experience. At a glance, the 17 capabilities are:

1. Universal Chat and Agent Relay, so one conversation can move between the native agents you have bound.
2. Bring Your Own Agent, with a Registry, an Agent Matchmaker that suggests which bound agent fits a job, and an Upgrade Shield.
3. A coding cockpit with an interactive terminal and Git and worktree views.
4. Durable Tasks with multi-agent delegation, Compare Agents, and a Mission Control canvas.
5. Persistent Coworkers: assistant identities with instructions, scoped context and approved responsibilities.
6. Computer interaction and a Workbench (files, browser preview, terminal, activity).
7. LiteSPM and a native package Marketplace for cross-agent capabilities.
8. Connections and app integrations through brokered, consent-scoped access.
9. Local Knowledge, Resources, RAG, and a Context Debugger that shows exactly what entered model context.
10. Routines and Automations with a local recurring host.
11. Memory and Knowledge Learning that stays review-gated.
12. A Library of Artifacts you can preview, save and export.
13. Intelligent UI with layered presentation and interactive artifacts.
14. Mission Control plus a persistent, pinnable Workspace Dashboard.
15. A Needs You inbox that gathers everything waiting on you.
16. Media: image understanding where the agent supports it (voice and video are later scope).
17. A desktop Runtime with honest Settings, security, data portability, evidence and a local-first commercial model.

Whether every one of these is covered today, and why a few are partial or deferred, is answered capability by capability in the full product scope below.

## The three objects you work with

Everything in LiteCowork is one of three things:

| Object | What it is |
|---|---|
| Conversation | Human-facing chat history and turns. Ordinary chat needs no Task and no Coworker, and a Conversation may use different native agents over time. |
| Task | Durable, outcome-oriented work with an accepted plan, steps, attempts, effects, artifacts and evidence. Task creation is always explicit; a Conversation never silently becomes a Task. |
| Coworker | A persistent assistant identity with instructions, scoped context and approved responsibilities. A Coworker is a configuration, not a resident running process. |

## Honest completion: qualification before claim

LiteCowork never dresses up a hope as a result. An agent saying it finished is not proof that it did.

- **Reported** is what the agent or a provider event says happened.
- **Observed** is what LiteCowork actually saw (a route responded, a file changed on disk).
- **Verified** is a completed, evidence-backed outcome qualified for the exact build, operating system, agent and provider version, and authentication route that produced it.

A mounted route, a package listing, a probe, a UI tile or a passing structural check is none of these on its own. **Unknown is not success:** where LiteCowork cannot confirm an outcome it says so, and surfaces that are not qualified are shown explicitly unavailable rather than faked.

## Full product scope

Are all the capabilities covered or not, and if not, why? The single, capability-by-capability answer, with a COVERED, PARTIAL or DEFERRED verdict and a one-line reason for each of the 17 capabilities, plus the UI shell, the deferred V1.1/V1.x/V2 boundaries, and the 12 qualification journeys, is in:

- [V1 product feature coverage](docs/V1-PRODUCT-FEATURE-COVERAGE.md)

In short: all 17 capabilities are owned by an accepted or in-review package; none is unowned. The remaining gaps are depth (exact qualification) and a few deliberately deferred surfaces, not missing scope. The native-agent adapter is defined as an Agent Client Protocol (ACP) client with a per-provider launch catalog, which delivers chat, bring-your-own-agent, the coding cockpit and the Needs You inbox through one qualified path.

## Product principles

- **BYOA first:** preserve native agent behavior, authentication, models, prompts, tools, private sessions and agent-owned controls.
- **Conversation first:** ordinary chat does not require a Task or Coworker.
- **Durable work:** outcome-oriented work may become an explicit Task whose state survives replaceable workers and daemon restarts.
- **Honest completion:** agent self-report is not verification. Effects, artifacts and acceptance evidence determine whether work is complete.
- **Local ownership:** V1 execution is local to the user's desktop unless the user explicitly connects a provider. Laptop-off cloud continuation is later scope.
- **Shared capabilities:** LiteSPM is the package lifecycle authority across compatible agent hosts; LiteCowork separately governs assignment, grants and mediated use.
- **One workspace:** chat, files, work, Coworkers, Routines, Automations and deliverables share one Workspace and history model.
- **Progressive disclosure:** plain-language outcomes are primary; agent options, policy, process and evidence are inspectable when needed.

## Where to read next

Product and design, in order:

1. [Product contract](docs/PRODUCT.md)
2. [V1 product release contract](docs/V1-PRODUCT-RELEASE-CONTRACT.md)
3. [V1 product feature coverage](docs/V1-PRODUCT-FEATURE-COVERAGE.md)
4. [V1 desktop screen and interaction contract](docs/V1-UI-SCREEN-CONTRACT.md)
5. [Feature SDD register](docs/V1-FEATURE-SDD-REGISTER.md) and [specifications](specs/README.md)

Current truth and status:

- [Current implementation and release status](docs/IMPLEMENTATION-STATUS.md)
- [Contradiction and alignment register](docs/CONTRADICTION-AND-ALIGNMENT-REGISTER.md)
- [Acceptance and E2E plan](docs/ACCEPTANCE-AND-E2E-PLAN.md)

## Release posture

**Pre-release development. V1 is not release-qualified.** The application and daemon source are present in this repository. The current implementation boundary, the available evidence and the ordered blockers are maintained in [docs/IMPLEMENTATION-STATUS.md](docs/IMPLEMENTATION-STATUS.md). Do not treat any capability as Verified without the exact-build evidence recorded there.

## For contributors

Architecture, build and the documentation workflow are in [CONTRIBUTING.md](CONTRIBUTING.md), which covers the Rust workspace and daemon (apps/litecoworkd), the Tauri v2 / React desktop app (apps/litecowork-ui), the domain crates (crates/), the normative contracts (docs/), the feature SDD packages (specs/) and the validators (.specify/). Before claiming a feature works, trace its screen and control to the typed Tauri command, the authenticated Operator route, the service owner, the storage or effect transaction, the provider integration, the stream or projection, and the recovery behavior, and record CODE, SERVED, INTEGRATED and QUALIFIED separately.
