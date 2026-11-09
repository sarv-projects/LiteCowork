# Data Model - Rich Presentation, Intelligent UI and Generated Media

**Status:** IN_REVIEW - authored for owner review; does not authorize implementation. No migration is planned; see section 6.

Canonical definitions: [docs/RICH-RESPONSE.md](../../docs/RICH-RESPONSE.md), [docs/PRESENTATION-RUNTIME.md](../../docs/PRESENTATION-RUNTIME.md), [docs/STATE-MACHINES.md](../../docs/STATE-MACHINES.md) "RichPresentation", [docs/schemas/rich-presentation.schema.json](../../docs/schemas/rich-presentation.schema.json), [docs/ARTIFACTS-EVIDENCE.md](../../docs/ARTIFACTS-EVIDENCE.md).

## 1. Existing durable records (reused)

| Record | Location | Fields |
|---|---|---|
| `rich_presentations` | migration `rich_presentation_optional_v12` (`crates/storage-sqlite/src/lib.rs`) | presentation_id, workspace_id, conversation_id, message_id, schema_version, renderer_contract_version, semantic_content_digest, document_digest, document_size_bytes, blob ref, producer_agent_session_id, created_at |
| Blob with purpose `RICH_PRESENTATION` | `crates/storage-core/src/lib.rs` | content-addressed document bytes, media type `application/vnd.litecowork.rich-presentation+json` |
| Artifact, ArtifactVersion | `crates/storage-core/src/artifact.rs` (`ArtifactVersionRecord { artifact_id, version, resource_revision_id, input_refs, content, provenance, created_by_attempt, verification_refs, created_at }`) | Generated media is a new Artifact version; create path from Feature 007 |
| `conversation_messages` | Feature 001 | Authoritative semantic text; never modified here |

## 2. Document shape (V1 profile, accepted target after T02)

| Field | Bound |
|---|---|
| Document bytes | 1 to 524,288 (512 KiB), RFC 8785 canonical |
| root_blocks | at most 200, total block paths at most 200, depth at most 8 |
| JSON values | at most 30,000, JSON depth at most 32 |
| accessibility_summary | at most 4,000 characters, must match a semantic line |
| block kinds | TEXT_SLICE, LAYOUT, CARD, CALLOUT, TABLE, CHART, SAFE_CALCULATOR |
| citations, actions | empty |
| block_provenance | one per block path, origin SEMANTIC_MESSAGE or MODEL_INTENT, all ref arrays empty |
| CHART | chart_kind BAR, LINE or AREA; binding to a semantic TABLE slice; accessible_summary at most 2,000 |
| SAFE_CALCULATOR | inputs at most 8 (id, label at most 200, initial value bound to a table cell), formula_ast at most 64 nodes and depth 12, output_format NUMBER, INTEGER, PERCENTAGE or CURRENCY_USD, fallback_text at most 2,000 |

## 3. Lifecycle

A RichPresentation is immutable. Identity is (message_id, presentation_id); the binder derives presentation_id deterministically from the message id and binder version, so a replay is idempotent and a binder upgrade creates a new presentation. States: absent, published, superseded (a newer presentation for the same message; the desktop shows the newest valid one). Rejected documents are not stored; a `rich.presentation.rejected.v1` event records ids and reason only.

Calculator state is view-local React state: created on render, discarded on unmount, never persisted or transmitted.

Interactive Artifacts keep the INTERACTIVE-ARTIFACT-RUNTIME lifecycle; in V1 the only reachable state is PREVIEW_ONLY.

## 4. Generated media

| Step | Record |
|---|---|
| Sniff and bound | none (rejections emit an event with reason) |
| Store bytes | blob `blobs/<sha256>` |
| Create Artifact version | Feature 007 create transaction: Artifact, ArtifactVersion with `provenance = {conversation_id, turn_id, agent_session_id, source: "HARNESS_FILE", digest, media_type, width, height}`, association to the Conversation |

Allowed types: PNG, JPEG, WebP, GIF; at most 20 MiB; at most 16,384 pixels per side. SVG is rejected in V1.

## 5. Privacy, retention and deletion

Presentation documents are derived from semantic messages and follow their retention and the Conversation delete fences (Feature 014); deleting a message deletes its presentations. Media Artifacts follow Artifact retention. Logs and events carry ids, digests and reasons only.

## 6. Migration and compatibility

No new migration. The only storage change is the document cap constant (1 MiB to 512 KiB) for new publishes; no production rows exist today. Renderer contract version 1 documents remain valid.
