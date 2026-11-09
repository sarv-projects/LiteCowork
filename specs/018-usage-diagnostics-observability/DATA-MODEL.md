# Data Model - Usage, Diagnostics and Observability

**Status:** IN_REVIEW - pending owner acceptance of B-42; does not authorize implementation; not a claim of implementation or qualification.
**Feature:** 018 | **Authority:** [V1-TELEMETRY-USAGE-RESILIENCE-LLD.md](../../docs/V1-TELEMETRY-USAGE-RESILIENCE-LLD.md), [OBSERVABILITY.md](../../docs/OBSERVABILITY.md), [STORAGE.md](../../docs/STORAGE.md), [SCHEMAS.md](../../docs/SCHEMAS.md), Feature 001 B-10.

Domain events, audit records, usage records, operational logs and crash records are separate data classes. This package must not create a parallel Conversation, turn or AgentBinding record.

## 1. Usage record (written by Feature 001, defined by this package's normalization)

One row per settled turn attempt (turn id plus retry ordinal), written inside Feature 001's final-response transaction. Proposed fields:

| Field | Type | Rule |
|---|---|---|
| usage_record_id | TEXT PK | Derived from (turn_id, retry_ordinal, provider_response_id or "none") so replay writes nothing new |
| workspace_id, conversation_id, turn_id | TEXT | Same-owner scope |
| retry_ordinal | INTEGER | From Feature 001 B-04 caveat |
| agent_binding_id | TEXT | Turn-pinned binding |
| model_id | TEXT NULL | Only from a typed adapter field |
| provider_response_id | TEXT NULL | Only from a typed adapter field |
| attribution | TEXT | PROVIDER_REPORTED, ESTIMATED_LOCAL, UNKNOWN |
| confidence | TEXT | Mapped: NATIVE_REPORTED, TOKENIZER_ESTIMATE, UNKNOWN |
| completeness | TEXT | COMPLETE, PARTIAL, UNKNOWN |
| input_tokens, output_tokens | INTEGER NULL | NULL means UNKNOWN, never zero |
| cache_read_input_tokens, cache_write_input_tokens, reasoning_output_tokens | INTEGER NULL | Subsets; never added to totals |
| outcome | TEXT | COMPLETED, FAILED, CANCELLED |
| observed_at | TEXT | RFC 3339 UTC at settlement |

No content, prompt, path, credential or cost field. Cost lanes in V1 are derived as null ("Not priced") because no PriceBook exists.

Index (this package, if 001 does not add it): `(workspace_id, observed_at)` covering `agent_binding_id, conversation_id, attribution, input_tokens, output_tokens`.

## 2. Projections (read models, rebuildable)

| Projection | Source | Shape |
|---|---|---|
| UsageSummaryView | usage records in window | measured lane (NATIVE_REPORTED only), estimate lane, unknown count, coverage, costs by currency (nullable), last_observed_at |
| UsageItemsPage | usage records grouped by Day, Conversation, AgentBinding or Model | label, requests, tokens per type with coverage, source, time range |
| ConversationUsageSummaryView | records of one Conversation across bindings | existing OpenAPI `ConversationUsageSummary` |
| HealthProbeView | live probes | probe, state, observed_at, stale_after_ms, reason_code |

Projections are never persisted as aggregates; rollups never read other rollups.

## 3. Quota observation

Existing OpenAPI `QuotaObservation` (served route returns `null`). A producer is a Feature 002 AgentModule adapter; this package only displays it. Never converted to money or summed with usage.

## 4. Diagnostic files

| File class | Location | Bound | Content |
|---|---|---|---|
| Operational log | `<data-dir>/diagnostics/log/litecoworkd-YYYYMMDD-N.jsonl` | 5 files x 5 MiB (proposed); ring of 2,000 entries in memory | `DiagnosticEntry` only: time, level, component, event code, reason code, correlation ids, duration |
| Crash record | `<data-dir>/diagnostics/crashes/<utc>-<process>.json` (daemon) and the Tauri app data directory (desktop) | newest 20; 16 KiB each | build, OS, time, source location, symbol-only frames, redacted class |
| Support bundle | owner-chosen path | 10 MiB | selected allowlisted categories plus manifest of SHA-256 digests |
| Usage export | owner-chosen path | 10 MiB | leaf metadata for the current filter |

Directories are owner-only (0700 on Linux; owner-only DACL on Windows). Clearing logs or crash records never deletes domain events, audit records or Evidence.

## 5. Events

No new DomainEvent. Usage records are written by Feature 001's settlement, which already emits its turn events. Diagnostics are not domain facts.

## 6. Retention

Usage records follow Workspace data retention and are carried in backups (Feature 017) as Workspace-scoped rows. Logs rotate by size; crash records keep the newest 20. Bundles and exports are owner files and are never auto-deleted.

## 7. Migration

At most one forward-only index migration. No migration ships in IN_REVIEW.
