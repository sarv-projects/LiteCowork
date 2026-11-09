# Errors and recovery - Desktop Shell IPC Settings and Runtime Health

**Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.

Daemon crash shows disconnected, not an empty successful workspace. Restart creates a new incarnation and reconciles hosts per RUNTIME-LIFECYCLE. Drain stops admission before claiming stopped.

Denial, duplicate replay, timeout-after-dispatch, cancel, crash and restart are all required before QUALIFIED. Unknown external results reconcile before retry.

## Revision 1.0 stop/update and Settings recovery

- Stop preview with any unsettled or UNKNOWN item returns BLOCKED or UNKNOWN; the UI lists the items and offers Retry preview. Stop never reports STOPPED early.
- Update without positive quiescence is refused with the same item list; simultaneous stop and update requests resolve to one admitted drain.
- Keep-awake: OS refusal or unsupported mode shows UNSUPPORTED/UNKNOWN; the assertion is released on drain and cleared at restart.
- Workspace rename version conflict shows the latest name and preserves the draft; duplicate replay returns the original receipt.
- IPC DACL creation failure stops startup with a typed error; there is no loopback fallback.
