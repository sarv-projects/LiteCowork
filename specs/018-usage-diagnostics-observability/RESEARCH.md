# Research and Decision Log - Usage, Diagnostics and Observability

**Feature:** 018 | **Status:** IN_REVIEW - pending owner acceptance of B-42; does not authorize implementation; not a claim of implementation or qualification.
**Checked:** 2026-11-09 - source audit of the checkout on branch `docs/audit-competitor-superset`; external sources are not web-verified in this pass.

## 1. Repository sources inspected

| Source | Path | Finding | Limit |
|---|---|---|---|
| Usage LLD | `docs/V1-TELEMETRY-USAGE-RESILIENCE-LLD.md` | Inference leaf model, two ledgers (usage leaves, QuotaSnapshot), dedupe precedence, honest empty state, support bundle preview rule, candidate modules under `apps/litecoworkd/src/telemetry/` | Normative target; states no served API |
| Observability | `docs/OBSERVABILITY.md` | Separate data classes; correlation fields; never log secrets | Metrics list is aspirational |
| Feature 001 | `specs/001-conversation-session-supervisor/SPEC.md` B-10 | Accepted: `attribution_source` PROVIDER_REPORTED / ESTIMATED_LOCAL, `max(1, utf8_bytes / 4)` | Names differ from LLD `UsageConfidence` |
| Quota route | `apps/litecoworkd/src/agent_quota_observation_operator.rs` | Served, owner-scoped, `no-store`, returns `null` | No producer |
| Quota UI | `apps/litecowork-ui/src/agents/QuotaObservation.tsx`, `agent-quota-api.ts`; Tauri `agent_quota_bridge.rs` | Honest null, expired and future handling | Reusable |
| OpenAPI | `docs/schemas/operator-api.openapi.yaml` (`getConversationUsage`, `listTaskUsage`, `ConversationUsageSummary`, `UsageObservation`, `QuotaObservation`) | Shapes exist | Target-only |
| Adapter | `apps/litecoworkd/src/agents/opencode_server.rs` | No token, cost or usage parsing | O-01 |
| Diagnostics | `eprintln!` in `main.rs`, `operator.rs`, `conversation_operator.rs`, `agents/supervisor.rs`, `backup_operator.rs` | 12 sites total including one test | No structured sink |
| Dependencies | `apps/litecoworkd/Cargo.toml`, `Cargo.lock` | `tracing` only transitive; `reqwest` is a daemon dependency (diagnostics must not use it) | n/a |
| Settings | `docs/V1-UI-SCREEN-CONTRACT.md` "Usage and monitoring dashboard", categories 14 and 16; `docs/V1-SETTINGS-CONTRACT-LEDGER.md` rows `inspectUsageTotals` to `exportUsageReport`, `showVersion` to `openSupportDocs`; `apps/litecowork-ui/src/settings/SettingsView.tsx` | Detailed target; four categories served | Ledger is proposed design |
| Status | `docs/IMPLEMENTATION-STATUS.md` "Usage and telemetry" | No UsageSummary, UsageItems or QuotaSnapshot projection served | n/a |

## 2. Pinned external sources

| Source | Version or SHA | License | Use | Decision |
|---|---|---|---|---|
| nightly-labs/openbot | `6759fa3ee857f4f4966865e12de99ae84ab6de07` ([REFERENCE-IMPLEMENTATIONS.md](../../docs/REFERENCE-IMPLEMENTATIONS.md) REF-OPENBOT; [study](../../docs/OPENBOT-REFERENCE-IMPLEMENTATION-STUDY.md)) | PolyForm Noncommercial 1.0.0 | Usage dashboard layout, provider-specific readers, cumulative counter checkpoints (`docs/architecture/analytics.md`, `src/renderer/src/features/usage/AgentUsagePanel.tsx`) | Prototype layout only; reject copying code (license); LiteCowork keeps the stricter two-ledger model |
| OpenTelemetry GenAI semantic conventions | not pinned (link in the LLD) | Apache-2.0 | Vocabulary for request phases and token attributes | Reference vocabulary only; no exporter in V1 |
| OmniRoute | not pinned (links in the LLD) | not reviewed | Request telemetry and circuit breaker concepts | Reject as a gateway; the LLD requires pinning before copying any algorithm |
| `tracing` crate | present transitively in `Cargo.lock` | MIT | Optional base for the typed log (F-01) | Allowed with an allowlist layer |

## 3. Alternatives considered

| Topic | Alternative | Decision |
|---|---|---|
| Log redaction | Regex scrubbing of free-form messages | Rejected: brittle; typed entries without free text instead |
| Crash reporting | Upload to a crash service | Rejected: crash reports stay local (owner rule) |
| Usage source for V1 | Opt-in reader of native session files | Deferred (O-04); typed adapter counters first |
| Quota wire name | New `QuotaSnapshot` schema | Rejected: keep existing `QuotaObservation` (O-02) |
| Timezone bucketing | `chrono-tz` in the daemon | Not preferred: UI sends UTC boundaries (F-02) |
| Cost | Ship a PriceBook in V1 | Rejected: "Not priced" until a price source is accepted |

## 4. Verified source facts (inputs to tasks)

1. No usage record anywhere in Rust source (T03, T05; Feature 001 write task).
2. OpenCode adapter has no counter parsing (T04).
3. Quota route returns `null` by design (T12 reuses the honest UI).
4. 12 `eprintln!` sites and no panic hook (T07, T08).
5. No E2E-xx covers usage or diagnostics (O-03).

## 5. Performance baseline (to be filled by 018-T14)

| Fixture records | OS/build | Summary p50/p95 | Items page p50/p95 | Query plan uses index |
|---|---|---|---|---|

## 6. Unresolved questions

See SPEC section 17 (O-01 to O-05).

## DeepSeek Harness proposals (PROPOSED, owner review required)

Research note, not a decision. Source: [deepseek-ai/deepseek-harness](https://github.com/deepseek-ai/deepseek-harness/tree/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159) at the pinned SHA (MIT, Copyright (c) 2026 DeepSeek). Local read-only clone `repo_take/deepseek-harness` is gitignored and never staged. Upstream paths below were checked to exist at that SHA on 2026-11-10.

Read-depth limits from the study (do not over-trust): `scripts/` about 58% read, tests about 5% read, installer UI text partly unread. See [desktop study](../../docs/DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Sections 0 and 15 before turning any item into a design.

- **P-33 Bounded rotating JSONL journal with partial-line repair, disable-on-uncertain-write, and an isolated support-bundle collector.** Class **ADOPT (algorithm port)**. Upstream: [src/operation-journal.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/operation-journal.ts), [src/operation-history.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/operation-history.ts), [src/operation-report-job.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/operation-report-job.ts), [src/crash-report.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/crash-report.ts). Study: [desktop study](../../docs/DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Sections 8 and 14 (row P-33). Lesson: `operation-journal.ts` imports only `node:fs/promises` and `node:path`, so the algorithm (two segments, record cap, queue cap, partial-line repair, bounded drain) ports to Rust directly. It fits the rule of no unbounded logs (T07 of package 018) and records must stay redacted at the producer. Reuse: Algorithm only, reimplement in Rust (or in the LiteCowork TypeScript tree for UI logic); the upstream source has only Node imports so the algorithm ports directly, but the file itself is not imported. If any text or structure is carried over, MIT applies (upstream LICENSE, "Copyright (c) 2026 DeepSeek"): keep that copyright line and the MIT permission notice in the third-party notices produced by task T13 of package 019, and cite the upstream path and SHA `fcc0ad46` in the code comment. Where it applies: T07, T08 and T10 of package 018; T15 of package 011. Deferral: Not started. Rust daemon crate; unit tests run on Linux. The reader semantics (dedupe by device and inode, never parse an unterminated tail) are acceptance cases.
- **P-44 Streaming secret redactor for subprocess output that holds back a buffer whose prefix could still become a secret.** Class **ADOPT (algorithm port)**. Upstream: [scripts/packaging-run.mjs](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/packaging-run.mjs), [scripts/packaging-step.mjs](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/packaging-step.mjs). Study: [desktop study](../../docs/DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Sections 8 and 14 (row P-44). Lesson: Whole-chunk replacement leaks a secret split across chunks. `packagingOutputRedactor` is a small pure algorithm, so it ports to Rust directly. It sits beside the masking in `scripts/v1_secret_scan.py` and the secret canary rule (T03 of package 017). Reuse: Algorithm only, reimplement in Rust (or in the LiteCowork TypeScript tree for UI logic); the upstream source has only Node imports so the algorithm ports directly, but the file itself is not imported. If any text or structure is carried over, MIT applies (upstream LICENSE, "Copyright (c) 2026 DeepSeek"): keep that copyright line and the MIT permission notice in the third-party notices produced by task T13 of package 019, and cite the upstream path and SHA `fcc0ad46` in the code comment. Where it applies: T07 of package 018; T03 of package 017. Deferral: Not started. Rust unit tests with a secret split at every byte offset run on Linux.

