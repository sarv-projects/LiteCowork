# Error and Recovery Contract - Feature 016

**Status:** IN_REVIEW - authored for owner review; does not authorize implementation.
Canonical codes: [docs/schemas/error-codes.schema.json](../../../docs/schemas/error-codes.schema.json). Recovery semantics: [docs/FAILURE-RECOVERY.md](../../../docs/FAILURE-RECOVERY.md).

## 1. Rejection reasons (`PRESENTATION_REJECTED.reason`, accepted target)

| Reason | Trigger | User-visible result |
|---|---|---|
| `TooLarge` | document over 512 KiB | semantic text |
| `NotCanonical` | bytes are not RFC 8785 canonical | semantic text |
| `DigestMismatch` | document or slice digest differs | semantic text |
| `UnknownKey` | key outside the closed object | semantic text |
| `UnsupportedBlock` | kind outside the V1 allowlist | semantic text |
| `DepthExceeded`, `TooManyBlocks` | depth over 8 or over 200 paths | semantic text |
| `BindingMismatch` | TABLE or CHART does not match the semantic table | semantic text (chart: table fallback if the table itself is valid) |
| `NonFiniteNumber` | NaN or Infinity in a value | semantic text |
| `CalcTooComplex`, `CalcUnknownVariable` | CalcAst over limits or unbound variable | semantic text |
| `ProvenanceInvalid` | provenance origin or refs not allowed | semantic text |

Media: `MEDIA_TYPE_UNSUPPORTED` (SVG, unknown, magic-byte mismatch, URL-only), `MEDIA_TOO_LARGE` (over 20 MiB or 16,384 pixels per side). No Artifact is created; the answer is unchanged.

## 2. Failure and crash matrix

| Window | Required behaviour |
|---|---|
| Binder throws or times out | Turn already settled; nothing published; counter increments |
| Crash after presentation blob write, before row commit | Orphan blob removed by the blob sweep; next binder run republishes idempotently |
| Crash after media blob write, before Artifact commit | Orphan blob removed; harness file can be re-ingested |
| Duplicate publish (same id, same digest) | Stored record returned |
| Same id, different digest | Conflict; no write |
| Two publishes race for one message | One commits; the other gets the conflict or the stored record |
| Stream gap | `stream.resync_required`; desktop reloads the Conversation presentation; no duplicate messages |
| Desktop validator rejects a published document | Semantic text shown; a local diagnostic counter increments (no network report) |
| Calculator divide by zero or non-finite | "Cannot calculate with these values" plus fallback text |

## 3. Mandatory behaviour

Presentation never blocks, retries or changes a turn. Validation failure is never shown as an answer failure. No recovery path rewrites a semantic message.
