# LiteSPM integration study (2026-11-09)

**Status:** research and integration grounding for Feature 013. Not an owner contract. Authority for LiteCowork behavior stays with ARCHITECTURE.md, AGENTS.md section 6 and the accepted packages. Authority for LiteSPM behavior stays with the LiteSPM repository's own ARCH and STATUS documents, cited below by file and line.
**Source:** [sarv-projects/LiteSPM](https://github.com/sarv-projects/LiteSPM) cloned read-only into `repo_take/LiteSPM` (gitignored, never imported, per AGENTS.md section 7). HEAD `84d5faba75bc621a651e0cec0f831e936bb2082d`, `git describe` = `v0.6.0-143-g84d5fab`, dated 2026-11-09. npm package version `0.6.0`. License Apache-2.0. Go project, 1,039 tracked files, 426 Go files, with a 40-document `ARCH/` set and a `STATUS.md` truth baseline.
**Why this exists:** Feature 013 previously guessed at the LiteSPM bridge because the marketing site ([lite-spm-nine.vercel.app](https://lite-spm-nine.vercel.app/)) states the product but not the wire contract, and three conflicting pins were in circulation (9f42b25d / 111fd9b8 / f77a3a3). The repository publishes the exact contract. This study records it so 013 is pinned from the published source, never inferred from CLI output or marketplace listings (AGENTS.md section 6).

## 1. What LiteSPM actually is

LiteSPM is a standalone, local-first package manager for AI-agent capabilities (MCP servers, agent skills, portable plugin components), written in Go, with its own daemon, its own SQLite state, its own secret vault, its own catalog index and its own MCP bridge. It is not a library LiteCowork links; it is a separate process LiteCowork talks to. Its own STATUS.md fixes a six-state evidence vocabulary (`DESIGNED`, `IMPLEMENTED`, `WIRED`, `TESTED`, `VERIFIED`, `SHIPPED`) that is almost identical in spirit to LiteCowork's TARGET/CODE/SERVED/INTEGRATED/QUALIFIED scale, and it holds itself to the same "files exist is at most IMPLEMENTED; tests exist is at most TESTED" discipline (`STATUS.md` lines 9-32). That shared honesty model is the single most important finding: the two projects can be reconciled truthfully because both refuse to call a mounted surface a working journey.

LiteSPM's own boundary statement matches LiteCowork's ownership split exactly: "LiteSPM's hosted catalog service is strictly a discovery index. It is never a secret store, execution environment, MCP proxy, or authentication authority. Downstream service credentials and runtime tool execution remain strictly on the user's local machine or with the user's chosen remote provider" (`ARCH/05-SECURITY.md:14`). This is the same line LiteCowork draws in AGENTS.md section 6: LiteSPM owns package discovery and lifecycle; LiteCowork owns scoped authorization for use.

## 2. The integration surface LiteCowork must bind (the pinned contract)

LiteSPM exposes two concentric interfaces. LiteCowork integrates at the **local IPC** layer, as a daemon client, exactly as the bridge shim does; it does not reimplement the shim.

### 2.1 Transport (matches LiteCowork's own IPC model)

- **JSON-RPC 2.0 over line-delimited frames**, newline-terminated, `MaxMessageSize = 16 MiB`, sequential integer request ids minted by the caller, id-less notifications (`ARCH/11-LOCAL-RUNTIME-IPC.md` section 3; `internal/ipc/protocol.go:25-26,110-166`).
- **Windows:** named pipe `\\.\pipe\litespm-daemon-<hex>` where `<hex>` is the first 12 hex chars of `SHA256(username)`; security descriptor `D:(A;;GA;;;OW)` grants the object owner only; the OS rejects a foreign SID before `Accept` (`ARCH/11` section 2.1; `internal/ipc/transport_windows.go:13-19`).
- **Linux/macOS:** unix socket `<RuntimeRoot>/litespm.sock`, directory `0700`, socket `0600`, with an authoritative kernel peer-credential check (`SO_PEERCRED` on Linux, `LOCAL_PEERCRED` on macOS) that refuses a peer uid different from the daemon's with `-32001 CodeUnauthorized` before any handler runs (`ARCH/11` section 2.2; `internal/ipc/transport_linux.go`, `transport_darwin.go`, `server.go`).
- This is the same OS-authenticated local-IPC posture LiteCowork's own daemon uses (peer uid check, owner-only DACL). The two trust boundaries are compatible; LiteCowork dials LiteSPM's endpoint as a same-user local client.

### 2.2 Protocol and handshake

- Handshake method `daemon.handshake`; wire types `HandshakeParams` / `HandshakeResult` (`internal/ipc/protocol.go:55-69`). Request fields: `clientVersion`, `clientKind` (`"cli"` or `"bridge"`), optional `hostId`, `pid`. Response: `daemonVersion`, `protocolVersion`, `pid`, `peerAuth {verified, method?, uid, pid?, reason?}`.
- **Protocol version string is `2026-07-28`** (24 references across `internal` and `cmd`; `ARCH/06-API-CONTRACTS.md:211`). The MCP side also negotiates `2026-07-28` in `initialize` and reports the real `buildinfo.Version` (`ARCH/14-BRIDGE-PROVIDER-MCP.md` section 1.2).
- Honest caveat to carry into 013: the handshake is defined but **not yet enforced or called by any production client** (`ARCH/11` section 3.1). 013 must treat the handshake as the contract to send, verify `peerAuth` and `protocolVersion`, and fail closed on a mismatch rather than assume LiteSPM gates it.
- Cancellation is `$/cancelRequest` with `{id}`, implemented both directions (`ARCH/11` section 3.2).

### 2.3 The 19 daemon methods and the 12 bridge tools

The daemon registers `daemon.handshake` plus 19 application methods (`ARCH/06` section 3.2; `cmd/litespm/main.go:1956-2744`): `tools.list`, `catalog.search`, `catalog.get_item`, `resolver.prepare_plan`, `install.execute`, `install.remove`, `skills.list`, `skills.load_body`, `skills.read_resource`, `capabilities.search`, `capabilities.describe`, `provider.probe`, `provider.invoke`, `invocation.get`, `invocation.cancel`, `host.detect_config`, `host.apply_setup`, `doctor.run_checks`, `system.status`.

The bridge shim advertises exactly 12 MCP tools and maps each to one daemon method (`ARCH/14` section 1.2). The naming split matters: MCP tool names are `verb_noun` (`search_catalog`), daemon methods are `domain.verb` (`catalog.search`), and `search_capabilities` -> `capabilities.search` is "what is installed and starts here", distinct from `search_catalog` -> `catalog.search` "what exists in the registry". LiteCowork binds the daemon methods, not the MCP tool names.

Status to respect (from LiteSPM's own STATUS, not assumed): `catalog.search`, `catalog.get_item`, `resolver.prepare_plan`, `install.execute` (for skills, MCP servers and pinned-source plugins), `install.remove`, `skills.*`, `invocation.get`, `invocation.cancel` are `WIRED`; `capabilities.search/describe` and `provider.invoke` "resolve" (reach the handler) with `provider.invoke` synchronous by contract; the providers table has no non-test writer, so nothing autostarts in production; provider health/circuit-breaking is `DESIGNED` only (`ARCH/14` sections 1.2, 2, 5).

### 2.4 Install plan and approval (binds cleanly to LiteCowork's plan-digest rule)

- `resolver.prepare_plan` builds, hashes and persists an `InstallPlan`; `install.execute` re-verifies it (`ARCH/06` section 2). `planId` matches `^plan_[0-9A-Za-z]{26}$`.
- **`planHash` is `sha256:` over the RFC 8785 canonical JSON of the execution fields** (excluding the volatile `planId`, `createdAt`, `expiresAt` and the `approval` record itself) (`ARCH/06` section 2.3). This is exactly the "approval bound to the plan digest" rule in AGENTS.md section 6: LiteCowork's 013 approval records bind to LiteSPM's `planHash`.
- The plan schema (`schemas/install-plan.schema.json`) carries `effects`, `requestedAccess`, `hostChanges`, `providerLaunches`, `sourceSnapshots`, `immutableRefs`, `selectedComponents`, `approval {approvalId, approvedBy, decision, decidedAt}`, `targetHost`, `targetScope`, `projectRoot`. LiteCowork previews these effects and host changes before the user approves.

### 2.5 Effect taxonomy and security model (aligns with LiteCowork invariants)

- LiteSPM classifies every plan action under a **17-action canonical effect taxonomy** (`filesystem.read/write/delete`, `process.spawn/signal`, `network.outbound`, `external.read/write/delete`, `credential.read/write`, `system.config.write`, `host.config.write`, `hook.execute`, `package.install`, `provider.start/stop`) with provenance tags `publisher_declared` / `curated` / `runtime_observed` (`ARCH/15-POLICY-APPROVALS.md` section 1). LiteCowork's Discover and grant UI should surface these effect declarations and their provenance rather than inventing its own.
- Egress is guarded centrally in `internal/egress`: HTTPS-only, SSRF refusal (RFC1918/ULA/link-local/metadata denied), credential-free URLs, bounded redirects (3 hops catalog/MCP, 5 artifact, 2 OAuth), destination re-validation per hop and rebinding-proof dialing (`ARCH/05` section 3). The remote-MCP policy adds same-origin redirect refusal. This matches LiteCowork's connector rules (TLS, SSRF, redirect policy, byte limits) in AGENTS.md section 6.
- Secrets never leave the local vault: a 32-byte master key in the OS keystore (DPAPI / Keychain / Secret Service), secret bytes AES-256-GCM in `secrets/vault.enc`, SQLite holding only `secret:<namespace>/<key>` references (`ARCH/05` section, `internal/secrets`). LiteSPM never puts a token in an agent prompt, which is the same rule LiteCowork enforces for Connections (capability 8).
- Grant drift: a capability grant binds `(capability_id, schemaFingerprint, casTreeDigest)` for local stdio or adds `(endpointOrigin, serverVersionDigest)` for remote; execution is refused on schema/tree/endpoint drift (`ARCH/14` section 4.1). LiteCowork's use-authorization should ride on top of, not duplicate, this binding.

### 2.6 Host adapters (the cross-agent write surface)

Six bespoke host adapters exist in `internal/host/`: `claude-code`, `codex`, `cline`, `opencode`, `grok-build`, `pi-agent`. Five support project-scope config paths (Claude Code `<root>/.mcp.json`, Codex `<root>/.codex/config.toml`, Grok Build `<root>/.grok/config.toml`, OpenCode `<root>/opencode.json`, Pi `<root>/.pi/mcp.json`); Cline refuses project MCP scope by its own docs (`ARCH/14` section 1.2 note). The marketing "50 hosts" figure is the catalog's host-target breadth; the bespoke adapters with verified project-scope writers are these six. LiteCowork's first qualified LiteSPM host should be one of the agents it already qualifies (OpenCode is the B-02 first route and has an adapter here), which keeps 013 and 002 in lockstep.

## 3. How LiteCowork integrates (reconciled with Feature 013)

The integration is a thin, pinned client, not a reimplementation:

1. **013 binds the daemon IPC, not the MCP bridge.** LiteCowork's `LiteSpmClient` (013-T04) dials the platform endpoint in section 2.1, sends `daemon.handshake`, checks `peerAuth.verified` and `protocolVersion == "2026-07-28"`, and calls the 19 methods directly. It does not run `litespm bridge stdio` for its own use; that shim is for arbitrary MCP hosts.
2. **Discover read-through** (013-T06) calls `catalog.search`, `catalog.get_item`, `capabilities.search`, `capabilities.describe` and `tools.list`, with single-flight and backoff per the cherry-studio pattern already noted. The "installable vs discovery-only" distinction the marketing site states is real in the data (`url` null vs present) and must be surfaced truthfully.
3. **Plan then approve then apply** (013-T07..T09) calls `resolver.prepare_plan`, records a LiteCowork approval bound to LiteSPM's `planHash`, then `install.execute` with the approval, and reads back the receipt. The effect taxonomy and `hostChanges` from the plan are what LiteCowork previews. `install.remove` strips only LiteSPM-written nodes (byte-exact restore is LiteSPM's guarantee, not LiteCowork's).
4. **Use authorization stays LiteCowork's** (013-T14, Feature 006): a package being installed by LiteSPM is not a Connection, a grant, or an invocation authority. LiteCowork governs assignment, Workspace/Coworker scope, and mediated use on top of LiteSPM's install.
5. **Honest status mapping:** LiteCowork must map LiteSPM's six states into its own scale and never upgrade them. A `DESIGNED` LiteSPM method (for example provider health, `ARCH/14` section 5) is TARGET in LiteCowork; a `WIRED` one is at most INTEGRATED until LiteCowork has its own qualification evidence.

## 4. What this resolves and what stays open

Resolved by this study:
- **The pin.** Feature 013 B-36 pins LiteSPM at `84d5fab` (`v0.6.0`), protocol `2026-07-28`, Apache-2.0, recorded in `specs/013 RESEARCH.md`. The three conflicting SHAs (9f42b25d / 111fd9b8 / f77a3a3) are superseded by this single published HEAD.
- **The wire names.** Taken from `ARCH/06` and `ARCH/11`, not inferred (AGENTS.md section 6 satisfied).
- **The safety contract.** LiteSPM's preview/backup/transaction/byte-exact-restore and its egress/secret/effect model are documented and align with LiteCowork invariants; 013 mirrors rather than re-specifies them.

Open, for the 013 implementation tasks (not this doc):
- The LiteSPM handshake is not enforced by LiteSPM today, so 013 must self-verify and fail closed.
- `provider.invoke` is synchronous and provider health is `DESIGNED` in LiteSPM; LiteCowork cannot present health it cannot observe.
- LiteSPM's release round trip (commit, CI, Cloudflare) can move HEAD; 013 re-reads the pin at implementation time and records the exact release it qualifies against, per LiteSPM's own "a probe is evidence only for the release it names" rule (`STATUS.md`).
- A full line-by-line code qualification (spawning a real daemon, exercising all 19 methods, confirming the host-adapter writes) is the work of 013-T02, 013-T04 and 013-T20, not of this study.

## 5. Method and limits

Read-only. The repository was cloned shallowly into the gitignored `repo_take/LiteSPM`; no code was copied into LiteCowork. Every claim cites a LiteSPM file and line at HEAD `84d5fab`. Where LiteSPM's own STATUS marks a behavior `DESIGNED` or "not yet true", this study repeats that label rather than upgrading it. The marketing site figures (50 hosts, 81 skill targets, ~69k listings) are the hosted catalog's breadth and are not a claim about LiteCowork's qualified surface.
