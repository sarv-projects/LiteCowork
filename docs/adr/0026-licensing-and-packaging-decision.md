# ADR 0026: Licensing and packaging decision (skeleton)

- **Status:** PROPOSED. Pending owner acceptance of register item B-44. This file is a skeleton created by 019-T02; it is not an accepted decision and 019-T13 stays BLOCKED until the owner accepts or amends it.
- **Owner:** Feature 019 (Product decision).

## Context

V1 is a local desktop product. A release build needs a recorded licensing and packaging decision (B-44, specs/019 SPEC section 5) before any installer is produced.

## Decision (proposed text from B-44)

1. LiteCowork Core local functionality ships without a mandatory subscription, account or licence key, and no licence-check network call exists.
2. The source licence is chosen by the owner. Proposed default: Apache-2.0, with a root `LICENSE` and matching `license` fields in every `Cargo.toml` and `apps/litecowork-ui/package.json`. Any other choice is recorded here with consequences.
3. Third-party notices are generated from `Cargo.lock`, `apps/litecowork-ui/src-tauri/Cargo.lock` and `pnpm-lock.yaml` and bundled as a resource; a dependency outside the reviewed licence allowlist fails the dependency audit.
4. Windows ships an NSIS per-user installer as the qualified V1 artifact plus an MSI for managed deployment; Linux ships deb; AppImage stays disabled.
5. No native agent harness, model or third-party agent binary is bundled.

## Consequences

- Release gate fails without this ADR in ACCEPTED state, a `LICENSE` and generated notices (AC-017).
- Packaging check fails when a bundle contains an `opencode` or `claude` executable.
