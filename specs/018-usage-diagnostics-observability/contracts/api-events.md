# API and Event Contract - Feature 018

**Status:** IN_REVIEW - pending owner acceptance of B-42; does not authorize implementation; not a claim of implementation or qualification.
**Canonical owners:** [API](../../../docs/API.md), [Schemas](../../../docs/SCHEMAS.md), [Telemetry LLD](../../../docs/V1-TELEMETRY-USAGE-RESILIENCE-LLD.md), [OpenAPI](../../../docs/schemas/operator-api.openapi.yaml).

## 1. Existing operations

| Operation | Path | State today | This package |
|---|---|---|---|
| `getAgentQuotaObservation` | `GET /agent-bindings/{agentBindingId}/quota-observation` | SERVED, returns `null` | Unchanged; displayed in the dashboard |
| `getConversationUsage` | `GET /conversations/{conversationId}/usage` | target_only | Serve with `ConversationUsageSummary`; `inference_count` null unless response ids are verified |
| `listTaskUsage` | `GET /tasks/{taskId}/usage` | target_only | Stays target-only until Feature 005 Attempts produce usage |

## 2. PROPOSED new operations

| Operation | Method and path | Query or body | Response |
|---|---|---|---|
| `getWorkspaceUsageSummary` | `GET /workspaces/{workspaceId}/usage-summary` | `start`, `end` (UTC, half-open), `bucket[]` (UTC boundaries, at most 400), `agent_binding_id[]`, `attribution[]`, `outcome[]` | `{window, measured: {input_tokens, output_tokens, cache_read_input_tokens, cache_write_input_tokens} (each nullable), estimated: {...} (nullable), record_count, unknown_record_count, coverage, costs_by_currency[], last_observed_at, buckets[]}` |
| `listWorkspaceUsageItems` | `GET /workspaces/{workspaceId}/usage-items` | window and filters, `grouping` (DAY, CONVERSATION, AGENT_BINDING, MODEL), `cursor`, `limit` (max 200) | page of `{label_ref, record_count, tokens by type with coverage, attribution mix, time_range}` |
| `exportWorkspaceUsageReport` | `POST /workspaces/{workspaceId}/usage-exports` | window, filters, `destination_ref`; Idempotency-Key | `{export_id, record_count, bytes, digest}` |
| `getBuildInfo` | `GET /diagnostics/build-info` | none | `{app_version, daemon_version, schema_version, os_family, os_version, arch, webview_version?}` |
| `listHealthProbes` | `GET /diagnostics/health` | none | `[{probe, state, observed_at, stale_after_ms, reason_code}]` |
| `listDiagnosticLogEntries` | `GET /diagnostics/log` | `level_min`, `component`, `cursor`, `limit` (max 500) | page of `DiagnosticEntry` |
| `listCrashRecords` | `GET /diagnostics/crashes` | none | newest-first list of crash record summaries |
| `previewSupportBundle` | `GET /diagnostics/support-bundle-preview` | `category[]` | `{categories: [{name, item_count, field_classes[]}], estimated_bytes, cap_bytes}` |
| `writeSupportBundle` | `POST /diagnostics/support-bundles` | `{categories[], destination_ref}`; Idempotency-Key | `{bundle_id, bytes, manifest_digest, categories[]}` |

All routes: authenticated local IPC, principal from the peer, Workspace routes check `X-Workspace-ID` and ownership, `Cache-Control: no-store`. No response contains prompt, message, completion, reasoning, file content, path (other than `~`-prefixed agent install paths in bundles), credential or handle.

`DiagnosticEntry`: `{at_utc, level, component, event_code, reason_code?, correlation: {workspace_id?, conversation_id?, turn_id?, operation_id?}, duration_ms?}`. There is no message field.

## 3. Tauri commands (PROPOSED)

`get_workspace_usage_summary`, `list_workspace_usage_items`, `get_conversation_usage`, `export_workspace_usage_report`, `get_build_info`, `list_health_probes`, `list_diagnostic_log_entries`, `list_crash_records`, `preview_support_bundle`, `write_support_bundle`, `record_desktop_crash` (desktop-local, no Operator call), and the existing `get_agent_quota_observation`.

## 4. Events

No new DomainEvent. Usage rows are written within Feature 001's settlement, whose existing turn events stand. Diagnostics, crash records and bundles are not domain facts and are never replicated.
