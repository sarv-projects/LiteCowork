# API, Command and Artifact Contracts - Feature 019

**Status:** IN_REVIEW - not owner-accepted; every interface below is PROPOSED in design and not in source.
**Canonical owners:** [ACCEPTANCE-AND-E2E-PLAN](../../../docs/ACCEPTANCE-AND-E2E-PLAN.md), [SECURITY](../../../docs/SECURITY.md), [RUNTIME-LIFECYCLE](../../../docs/RUNTIME-LIFECYCLE.md), Feature [011](../../011-desktop-shell-ipc-settings-runtime/SPEC.md) section 7.2.

This package adds no Operator route and no DomainEvent. Its interfaces are Tauri commands, files and CI artifacts.

## 1. Tauri commands (PROPOSED)

| Command | Input | Output | Rules |
|---|---|---|---|
| `check_for_update` | none | `UpdateCheckOutcome::UpToDate { current }` or `Available { version, notes_digest, channel }` | Fetch pinned HTTPS manifest; verify signature with embedded key; refuse downgrade and channel mismatch; never cached as "up to date" after a failure |
| `install_update` | `expected_version`, `request_id` (UUID) | `Refused { reasons }`, `InProgress { request_id }`, `Handoff { version }` | Single-flight; calls `check_update_quiescence` (Feature 011) first; downloads and verifies; requests authenticated stop; hands off to installer only after STOPPED |
| `check_update_quiescence` | owned by Feature 011 | `PositiveQuiescence` or `NotQuiescent(reasons)` | Called only by `install_update`; this package never computes quiescence |

`UpdateError` values: `ManifestUnreachable`, `SignatureInvalid`, `Downgrade`, `ChannelMismatch`, `AlreadyInProgress`, `QuiescenceUnavailable`, `RuntimeStopFailed`, `InstallerFailed`. They are typed results for the UI reducer, not entries in the Operator error-code registry, because no Operator route returns them.

## 2. Update manifest (PROPOSED, B-43)

```json
{
  "version": "1.0.1",
  "pub_date": "2026-12-01T00:00:00Z",
  "notes": "sha256 digest of release notes",
  "platforms": {
    "windows-x86_64": { "url": "https://<pinned-origin>/litecowork/stable/LiteCowork_1.0.1_x64-setup.nsis.zip", "signature": "<ed25519 signature>" }
  }
}
```

Only `windows-x86_64` appears in V1. The origin is fixed in `tauri.conf.json` `plugins.updater.endpoints`; redirects to another origin are refused.

## 3. Release artifacts

| Artifact | Row | Signature | Contents rule |
|---|---|---|---|
| `LiteCowork_<v>_x64-setup.exe` (NSIS, per-user) | Windows | Authenticode + timestamp | Shell, `litecoworkd-x86_64-pc-windows-msvc.exe`, LICENSE, THIRD-PARTY-NOTICES; no native agent binary |
| `LiteCowork_<v>_x64_en-US.msi` | Windows (managed) | Authenticode + timestamp | Same contents; qualified after NSIS |
| Updater archive + `.sig` | Windows | Ed25519 updater key | Produced by `createUpdaterArtifacts` |
| `litecowork_<v>_amd64.deb` | Linux | `SHA256SUMS` signed with release key | Shell, `/usr/bin/litecoworkd`, disabled systemd unit, LICENSE, notices |

## 4. CI artifacts and files

- `docs/release/QUALIFICATION-MATRIX.json` / `.md`: generated per release candidate (schema `release-matrix-row.schema.json`, new).
- Evidence bundle JSON: schema `release-evidence-bundle.schema.json` (new); index in `docs/release/evidence/INDEX.md`.
- `docs/release/V1-GO-NO-GO.md`: gate output digest and owner decision.

## 5. Events

None. Update attempts are recorded locally (DATA-MODEL UpdateAttempt) and surfaced through the Feature 018 diagnostics viewer; the daemon's own startup and migration events are owned by Storage and Runtime.
