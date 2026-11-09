# UI behavior - Settings Usage & Monitoring and About & Diagnostics (Feature 018)

**Status:** IN_REVIEW - pending owner acceptance of B-42; does not authorize implementation; not a claim of implementation or qualification. Binds UI to owner contracts; not pixels.
**Normative UI indexes:** [V1-UI-SCREEN-CONTRACT.md](../../../docs/V1-UI-SCREEN-CONTRACT.md) ("Usage and monitoring dashboard", Settings categories 14 and 16), [V1-SETTINGS-CONTRACT-LEDGER.md](../../../docs/V1-SETTINGS-CONTRACT-LEDGER.md) (rows `inspectUsageTotals`, `filterUsageDateRange`, `filterUsageAgent`, `inspectUsageConfidence`, `inspectCostPriceSource`, `inspectQuotaHealth`, `inspectFailureTimeline`, `exportUsageReport`, `showVersion`, `exportRedactedLogs`, `runProviderDiagnostic`, `copyBuildInfo`, `openSupportDocs`), [DESKTOP-SHELL-LLD.md](../../../docs/DESKTOP-SHELL-LLD.md).
**Wire:** [api-events.md](api-events.md).

The UI renders read-only projections and sends typed commands. It never computes authority or decides what is safe to export, and it renders errors by code.

## 1. Usage & Monitoring

| Control | Command | Enabled when | Notes |
|---|---|---|---|
| Quick range (Today, Last 7 days, Last 1 month, Last 3 months, Last 1 year, All time; advanced YTD, Custom) | local, then `get_workspace_usage_summary` | Always | `usage-window.ts` computes half-open UTC boundaries in the viewer's IANA zone |
| Scope (current Workspace / All Workspaces) | local | All Workspaces only with explicit selector and per-Workspace ownership | Default current Workspace |
| Agent filter | local | Always | Prefilled when opened from an Agent profile |
| Grouping (Day, Conversation, Agent, Model) | `list_workspace_usage_items` | Always | Remembered per Workspace |
| Row drill-down | `list_workspace_usage_items` | Row selected | Leaf metadata only; never content |
| Refresh | re-query | Always | Observation read only |
| Export | `export_workspace_usage_report` | Records exist in window | Native save dialog; bounded |
| Quota card | `get_agent_quota_observation` | Per binding | `QuotaObservationSummary`; null -> "Not reported by this agent" |

Headline metrics: measured input and output tokens (NATIVE_REPORTED only), estimated lane suffixed "(est.)", record count, unknown count, coverage with the named missing sources, costs by currency or "Not priced". Every chart has an accessible table with the same values.

## 2. About & Diagnostics

| Control | Command | Enabled when | Notes |
|---|---|---|---|
| Build info | `get_build_info` | Always | App, daemon, schema, OS, arch |
| Copy build info | local clipboard | Build info loaded | Copies exactly the displayed fields |
| Health | `list_health_probes` | Always | State plus age; Serving shown separately from execution readiness |
| Recent logs | `list_diagnostic_log_entries` | Always | Typed entries; no message text exists |
| Crash records | `list_crash_records` | Always | Newest first; local only |
| Support bundle | `preview_support_bundle` then `write_support_bundle` | Preview shown | Category checkboxes with counts and field classes; explicit Write; native save dialog |
| Run provider diagnostic | none in V1 | Only when an AgentModule declares a qualified probe | Otherwise shown as unavailable with the reason |
| Support docs | open local docs link | Always | No telemetry |

## 3. States

| Surface | States | Meaning |
|---|---|---|
| Usage summary | `LOADING`, `EMPTY`, `PARTIAL`, `READY`, `ERROR`, `DENIED`, `OFFLINE`, `STALE` | EMPTY text: "No measured usage yet; some agents do not expose token counts". PARTIAL names the missing sources |
| Token cell | value, `UNKNOWN`, `(est.) value` | UNKNOWN never renders as 0 |
| Cost cell | amount with currency, `Not priced` | Never `$0.00` by default |
| Coverage | percentage with source list, `UNKNOWN` | Never `100%` by default; only computed from records |
| Quota card | `NOT_REPORTED`, `NORMAL`, `LOW`, `EXHAUSTED`, `UNKNOWN (expired)`, `UNKNOWN (future timestamp)` | Never shown as money |
| Health probe | `READY`, `DEGRADED`, `BLOCKED`, `UNKNOWN`, `OFFLINE` | Age shown; stale becomes UNKNOWN |
| Bundle | `IDLE`, `PREVIEWING`, `WRITING`, `WRITTEN`, `FAILED(code)` | WRITTEN shows file name, size and manifest digest prefix |

## 4. Flow transitions

| Step | Surface | Transition |
|---|---|---|
| F1 | Open Usage | LOADING -> EMPTY, PARTIAL or READY |
| F2 | Change range or filter | LOADING -> new state; previous values cleared, not kept as stale numbers |
| F3 | Open from Agent profile | Agent filter applied |
| F4 | Daemon offline | OFFLINE with reconnect; no cached numbers presented as current |
| F5 | Open About after a crash | Crash record listed |
| F6 | Bundle | Preview -> deselect -> Write -> WRITTEN or FAILED |

## 5. Accessibility

Tables are keyboard navigable and sortable; each cell's accessible name includes its source and confidence (for example "Input tokens 12,400, estimated"). Charts are decorative to assistive technology when the table is present. Range buttons form a labelled group. Live regions announce load completion and bundle results politely. Color is never the only signal for estimated or unknown.
