# Errors and recovery - Native Agent Install, Auth and Binding

**Status:** ACCEPTED/LOCKED - owner-accepted 2026-11-09; authorizes implementation; not a claim of implementation or qualification.

Stale descriptor, expired auth, missing binary, unsupported version and cancelled native login are distinct typed failures. A failed probe does not delete a prior binding. Retry re-probes; it does not replay secret bytes. Crash mid-probe leaves the last committed binding and a failed observation.

Denial, duplicate replay, timeout-after-dispatch, cancel, crash and restart are all required before QUALIFIED. Unknown external results reconcile before retry.

Revision 1.0 additions (accepted target, not yet in source): an OpenCode offer that is not exactly v2.0.32, is partial or has unobserved host stop stays `compatible=false` with its reason in the offer constraints (AC-013); a descriptor request for a profile with no qualified module keeps `503 DEPENDENCY_UNAVAILABLE` (AC-014); a sign-in for an undeclared method fails as `AuthError::MethodNotDeclared` and a cancelled sign-in records no secret and leaves bindings unchanged (AC-003, AC-016). Recovery cases are tested in 002-T13.
