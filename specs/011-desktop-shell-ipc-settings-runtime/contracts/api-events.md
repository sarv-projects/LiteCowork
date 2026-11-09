# API and events - Desktop Shell IPC Settings and Runtime Health

**Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.

IPC is a transport for docs/API.md operations. Readiness and drain follow RUNTIME-LIFECYCLE. Settings classes follow the ledger.

Event names must already exist in [docs/EVENTS.md](../../../docs/EVENTS.md) or be added there with the schema in the same reviewed change. Revision 1.0 accepted target interfaces (not yet in source): `renameWorkspace`, the stop preview payload for `previewLocalRuntimeStop`, the update quiescence gate and the power-assertion observation. Any new event (for example a Workspace renamed event) is added to EVENTS.md and `docs/schemas/domain-event.schema.json` by 011-T01 in the same change; none exists yet.

Error codes must already exist in [docs/SCHEMAS.md](../../../docs/SCHEMAS.md) or be added with `docs/schemas/error-codes.schema.json`.
