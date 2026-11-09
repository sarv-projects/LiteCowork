# API and Event Contract - Feature 016

**Status:** IN_REVIEW - authored for owner review; does not authorize implementation. Links to global owners are normative.

## 1. Existing served routes

| Method and path | Handler | Behaviour |
|---|---|---|
| `GET /v1/rich-presentations/{presentation_id}` | `get_rich_presentation` in `apps/litecoworkd/src/presentation_operator.rs` | Workspace-owner check (403 `FORBIDDEN`), id validation (400 `INVALID_ARGUMENT`), read through `SqliteRichPresentationStore::read_rich_presentation` (500 `INTEGRITY_FAILURE` on failure), size recheck; returns the envelope with `workspace_id, conversation_id, message_id, presentation_id, schema_version, renderer_contract_version, semantic_content_digest, document_digest, document_size_bytes, document` |
| `GET /v1/conversations/{conversation_id}/presentation` | `get_conversation_presentation` | Conversation presentation snapshot with message and presentation references |
| `POST /v1/stream` | `subscribe_stream` | Frames `stream.ready`, `projection`, `stream.resync_required`; content type `application/vnd.litecowork.operator-stream+json` |

Tauri: `get_rich_presentation` in `apps/litecowork-ui/src-tauri/src/conversation_bridge.rs` checks the envelope with `valid_rich_presentation_response` before returning it to the WebView.

## 2. No new public route

Publishing is daemon-internal (binder after commit). Media ingestion is daemon-internal and creates Artifacts through Feature 007, whose routes serve them. This package adds no desktop-callable write route; a rendered element therefore has nothing to call.

## 3. Schema changes (accepted target, promoted by T02)

- `docs/schemas/rich-presentation.schema.json`: renderer contract version 2 adds `SAFE_CALCULATOR`; a V1 allowlist profile names TEXT_SLICE, LAYOUT, CARD, CALLOUT, TABLE, CHART, SAFE_CALCULATOR; CHART binding limited to a semantic table slice; `chart_kind` limited to BAR, LINE, AREA in the V1 profile.
- `docs/schemas/calc-ast.schema.json` (new): node kinds LITERAL, VARIABLE, BINARY_OP, UNARY_OP, FUNCTION; at most 64 nodes, depth 12.
- `docs/schemas/presentation-intent.schema.json`: unchanged; the binder consumes `TABLE`, `COMPARISON` and `CHART_FROM_RESULT` only when the result binding is a semantic table (other elements are ignored in V1).

## 4. Events

Existing: `rich.presentation.published.v1` (`presentation_id`, `conversation_id`, `message_id`, `schema_version`, `renderer_contract_version`, `semantic_content_digest`, `document_digest`, `document_size_bytes`, `producer_agent_session_id?`, `host_instruction_digest?`, `host_skill_refs[]`, `aggregate_version`).

Accepted target: `rich.presentation.rejected.v1` (`conversation_id`, `message_id`, `binder_version`, `reason`). Media Artifacts use the Feature 007 Artifact events.

## 5. Errors

Accepted target codes: `PRESENTATION_REJECTED` (internal, with `reason`), `MEDIA_TYPE_UNSUPPORTED`, `MEDIA_TOO_LARGE`. These never reach the user as a failed answer; they are counted and the semantic text is shown.

## 6. Security contract

No route, frame or event carries model HTML, scripts or URLs to be fetched. Documents carry no actions or citations in V1.
