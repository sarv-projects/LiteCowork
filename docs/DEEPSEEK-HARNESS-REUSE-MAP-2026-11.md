# DeepSeek Harness reuse map (2026-11-10)

**Status:** RESEARCH/TRACEABILITY. Not normative. This map records, for each pattern taken from the DeepSeek Harness, where it lives upstream, which LiteCowork task owns it, and whether LiteCowork would reuse the logic directly or adapt it. It does not authorize implementation, and it does not change any accepted package. No upstream code has been copied into LiteCowork as of this revision.

## Source, pin and license

| Item | Value |
|---|---|
| Repository | [deepseek-ai/deepseek-harness](https://github.com/deepseek-ai/deepseek-harness) |
| Pinned SHA | [`fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159`](https://github.com/deepseek-ai/deepseek-harness/tree/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159) (desktop 0.2.2-rc.1) |
| License | MIT, `Copyright (c) 2026 DeepSeek`, read from `repo_take/deepseek-harness/LICENSE` on 2026-11-10. |
| Local clone | `repo_take/deepseek-harness` (research only, gitignored, never staged, never imported; AGENTS.md section 7) |
| Studies | [Desktop shell deep-dive](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) (register A-63, proposals P-31 to P-46), [Plugin architecture deep-dive](DEEPSEEK-HARNESS-PLUGIN-ARCHITECTURE-DEEP-DIVE-2026-11.md) (register A-58, proposal P-27), [Reference matrix](REFERENCE-IMPLEMENTATIONS.md) |

## Why nothing is a verbatim drop-in

Upstream is an Electron and Node shell with a TypeScript Host. LiteCowork is a Tauri v2 shell with a Rust daemon and a React UI. A file is a **direct-reuse candidate** only when it is pure logic with no Electron or Node import; even then the daemon-side logic must be ported to Rust. Imports were checked for the two smallest candidates on 2026-11-10: [`apps/desktop/src/keyboard.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/keyboard.ts) imports `electron` and four workspace packages (not reusable as a file; reuse the rules only), [`apps/desktop/src/update-schedule.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/update-schedule.ts) imports a duration helper and the update coordinator types (reusable arithmetic, not the file), [`apps/desktop/src/operation-journal.ts`](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/operation-journal.ts) imports only `node:fs/promises` and `node:path` (the algorithm ports to Rust directly).

Reuse vocabulary used below:

- **ADOPT** (called PORT in earlier revisions of this file) - same algorithm and invariants, re-implemented in LiteCowork's language; the upstream file is still not imported. If any text or structure is carried over, add the MIT notice (`Copyright (c) 2026 DeepSeek` plus the MIT permission text) to the third-party notices produced by 019-T13 and cite the upstream path and SHA in the code comment.
- **ADAPT** - pattern only, reimplement: take the design and acceptance cases; the mechanics differ (Tauri, Rust, UDS and named pipe instead of Electron and loopback HTTP).
- **REJECT** - recorded so it is not re-proposed.
- **OWNER** - needs an owner decision first.

Before any ADOPT: re-read the upstream file at line level (the studies list what was only sampled, desktop deep-dive Sections 0 and 15), re-check the license at the root, and record the decision in the owning task.

## Proposals P-31 to P-46 (desktop shell)

Every path below is a full permalink at the pinned SHA (`apps/desktop/<path>`). Class vocabulary: ADOPT, ADAPT, REJECT, OWNER (defined above). All sixteen are PROPOSED only: not accepted, owner review required, nothing implemented from upstream. Each entry names the study section, the reason for the class, the reuse statement and the LiteCowork tasks (short form "T12 of package 011" is used on purpose so this file does not trip the duplicate-task-ID validator when quoted into a package).

### P-31 QuiescenceReport (one "interruptible work" predicate; unknown means busy; lock then learn; inspect once)

- **Class:** ADAPT
- **Upstream permalinks:** [src/host-process.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/host-process.ts), [src/quit-confirmation.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/quit-confirmation.ts)
- **Study:** [desktop study](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Sections 3 and 14 (row P-31)
- **Reason:** LiteCowork describes the same predicate in three places (T12, T13, T14 of package 011). Upstream shows the same facts must back stop, update, quit and keep-awake, and that unknown resolves to busy. The inputs differ (admitted turns, admitted Attempts, Terminal and Browser sessions, armed Automations, pending approvals, not Electron tasks), so it is an ADAPT, not a port.
- **Reuse statement:** Pattern only, reimplement. No upstream code or text is copied or imported (AGENTS.md section 7; upstream is Electron and TypeScript, LiteCowork is Tauri, Rust and React).
- **LiteCowork tasks:** T12, T13, T14 of package 011; T11 of package 019; T10 of package 010
- **Where proposed:** PROP-P31 in `specs/` package 011 TASKS.md (PROPOSED, not accepted)
- **Build, test and deferral:** Daemon half (`RuntimeLifecycleStore`, served `previewLocalRuntimeStop`) can be built and tested on Linux; the packaged quit path needs the Windows laptop. Not started.

### P-32 Per-request deadlines with defined meaning-on-timeout for destructive shell-to-daemon calls

- **Class:** ADAPT
- **Upstream permalinks:** [src/host-process.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/host-process.ts), [src/welcome-window.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/welcome-window.ts), [src/update-dialog.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/update-dialog.ts)
- **Study:** [desktop study](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Sections 2, 6 and 14 (row P-32)
- **Reason:** LiteCowork has IPC admission bounds (T04 of package 011) but no stated timeout meaning for stop, update and quit calls. The upstream rule (a timeout on a destructive gate means do not proceed) is reusable; the transport differs (UDS and named pipe, not loopback HTTP).
- **Reuse statement:** Pattern only, reimplement. No upstream code or text is copied or imported (AGENTS.md section 7; upstream is Electron and TypeScript, LiteCowork is Tauri, Rust and React).
- **LiteCowork tasks:** T04 and T13 of package 011; T07 of package 019
- **Where proposed:** PROP-P32 in `specs/` package 011 TASKS.md (PROPOSED, not accepted)
- **Build, test and deferral:** Not started. Needs the operator IPC and Tauri bridge decisions in package 011 first.

### P-33 Bounded rotating JSONL journal with partial-line repair, disable-on-uncertain-write, and an isolated support-bundle collector

- **Class:** ADOPT (algorithm port)
- **Upstream permalinks:** [src/operation-journal.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/operation-journal.ts), [src/operation-history.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/operation-history.ts), [src/operation-report-job.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/operation-report-job.ts), [src/crash-report.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/crash-report.ts)
- **Study:** [desktop study](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Sections 8 and 14 (row P-33)
- **Reason:** `operation-journal.ts` imports only `node:fs/promises` and `node:path`, so the algorithm (two segments, record cap, queue cap, partial-line repair, bounded drain) ports to Rust directly. It fits the rule of no unbounded logs (T07 of package 018) and records must stay redacted at the producer.
- **Reuse statement:** Algorithm only, reimplement in Rust (or in the LiteCowork TypeScript tree for UI logic); the upstream source has only Node imports so the algorithm ports directly, but the file itself is not imported. If any text or structure is carried over, MIT applies (upstream LICENSE, "Copyright (c) 2026 DeepSeek"): keep that copyright line and the MIT permission notice in the third-party notices produced by task T13 of package 019, and cite the upstream path and SHA `fcc0ad46` in the code comment.
- **LiteCowork tasks:** T07, T08 and T10 of package 018; T15 of package 011
- **Where proposed:** PROP-P33 in `specs/` package 018 TASKS.md (PROPOSED, not accepted)
- **Build, test and deferral:** Not started. Rust daemon crate; unit tests run on Linux. The reader semantics (dedupe by device and inode, never parse an unterminated tail) are acceptance cases.

### P-34 Updater coordinator spec (generation token, single-flight verbs, no auto-install, backoff ladder, refuse after unclean exit)

- **Class:** ADAPT (update-schedule arithmetic is an ADOPT candidate)
- **Upstream permalinks:** [src/update-coordinator.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/update-coordinator.ts), [src/update-schedule.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/update-schedule.ts), [src/update-presentation.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/update-presentation.ts), [src/update-attention.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/update-attention.ts)
- **Study:** [desktop study](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Sections 10 and 14 (row P-34)
- **Reason:** The LiteCowork UI reducer already exists (`apps/litecowork-ui/src/updates/`, 16 tests). Upstream adds the daemon-side rules: generation token, graceful-stop ladder with refuse-on-unclean-exit, 14-way failure taxonomy. `update-schedule.ts` is pure arithmetic over a duration helper and coordinator types, so only the arithmetic is a port candidate; the coordinator is an ADAPT because the Tauri updater differs from electron-updater.
- **Reuse statement:** Pattern only, reimplement. No upstream code or text is copied or imported (AGENTS.md section 7; upstream is Electron and TypeScript, LiteCowork is Tauri, Rust and React). For the schedule arithmetic only: If any text or structure is carried over, MIT applies (upstream LICENSE, "Copyright (c) 2026 DeepSeek"): keep that copyright line and the MIT permission notice in the third-party notices produced by task T13 of package 019, and cite the upstream path and SHA `fcc0ad46` in the code comment.
- **LiteCowork tasks:** T11, T12 of package 019; T14 of package 011
- **Where proposed:** PROP-P34 in `specs/` package 019 TASKS.md (PROPOSED, not accepted)
- **Build, test and deferral:** DEFERRED with B-43: the daemon and Tauri half cannot be exercised without a signed build and a feed.

### P-35 Sealed per-file runtime inventory written after signing, packaged smoke against a private data root, distinct unsigned artifact names

- **Class:** ADAPT
- **Upstream permalinks:** [src/runtime-tree.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/runtime-tree.ts), [scripts/electron-builder-config.mjs](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/electron-builder-config.mjs), [scripts/package-target.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/package-target.ts), [scripts/smoke-runtime.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/smoke-runtime.ts)
- **Study:** [desktop study](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Sections 11 and 14 (row P-35); Section 15 lists what stays unread
- **Reason:** Gives package 019 a byte-level "what we shipped" artifact that the evidence bundle (T15) can reference without a startup hashing cost. The smoke assertions in `scripts/smoke-runtime.ts` are only read to the header level, so the private-data-root design is not final until that file is read line by line.
- **Reuse statement:** Pattern only, reimplement. No upstream code or text is copied or imported (AGENTS.md section 7; upstream is Electron and TypeScript, LiteCowork is Tauri, Rust and React).
- **LiteCowork tasks:** T08, T09, T10, T14, T15 of package 019
- **Where proposed:** PROP-P35 in `specs/` package 019 TASKS.md (PROPOSED, not accepted)
- **Build, test and deferral:** Linux deb scripts exist (`scripts/qualify-linux-package.sh`); the signed run is DEFERRED with B-43. Read `scripts/smoke-runtime.ts` in full before this becomes a design.

### P-36 Publication discipline and rollback (plan validated before credentials, artifacts before manifest, reconcile unknown writes, feed-only rollback)

- **Class:** ADAPT
- **Upstream permalinks:** [scripts/desktop-upload-plan.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/desktop-upload-plan.ts), [scripts/desktop-publish.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/desktop-publish.ts), [scripts/desktop-publish-record.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/desktop-publish-record.ts), [scripts/desktop-release-store.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/desktop-release-store.ts), [scripts/desktop-rollback.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/desktop-rollback.ts), [scripts/desktop-cos.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/desktop-cos.ts), [scripts/desktop-purge-recovery.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/desktop-purge-recovery.ts)
- **Study:** [desktop study](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Sections 11 (11.4) and 14 (row P-36)
- **Reason:** The order of operations and unknown-write reconciliation are independent of the storage vendor. LiteCowork V1 distribution endpoint is undecided, so only the discipline is reusable; the COS and EdgeOne specifics are REJECT.
- **Reuse statement:** Pattern only, reimplement. No upstream code or text is copied or imported (AGENTS.md section 7; upstream is Electron and TypeScript, LiteCowork is Tauri, Rust and React).
- **LiteCowork tasks:** T10, T15 and T23 of package 019
- **Where proposed:** PROP-P36 in `specs/` package 019 TASKS.md (PROPOSED, not accepted)
- **Build, test and deferral:** DEFERRED with B-43 and B-44 (distribution endpoint undecided).

### P-37 Golden files for user-visible dialog copy and golden scenario lists for installer tests

- **Class:** ADAPT
- **Upstream permalinks:** [tests/expected/](https://github.com/deepseek-ai/deepseek-harness/tree/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/tests/expected), [tests/expected/windows-installer.json](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/tests/expected/windows-installer.json), [tests/expected/windows-uninstall.json](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/tests/expected/windows-uninstall.json), [src/locale.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/locale.ts)
- **Study:** [desktop study](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Sections 12 and 14 (row P-37)
- **Reason:** A reviewable golden for stop, update, recovery and failure dialog copy is a strong guard for the truthfulness rules (T19 of package 011). The files themselves hold DeepSeek strings and are not reusable; only the technique is.
- **Reuse statement:** Pattern only, reimplement. No upstream code or text is copied or imported (AGENTS.md section 7; upstream is Electron and TypeScript, LiteCowork is Tauri, Rust and React). If a scenario list from `windows-uninstall.json` or `windows-installer.json` is copied verbatim as a test outline, if any text or structure is carried over, MIT applies (upstream LICENSE, "Copyright (c) 2026 DeepSeek"): keep that copyright line and the MIT permission notice in the third-party notices produced by task T13 of package 019, and cite the upstream path and SHA `fcc0ad46` in the code comment.
- **LiteCowork tasks:** T19 of package 011; T16 and T08 of package 019
- **Where proposed:** PROP-P37 in `specs/` package 011 TASKS.md (PROPOSED, not accepted)
- **Build, test and deferral:** UI golden tests can run on Linux; installer goldens wait for the Windows laptop.

### P-38 Login-shell PATH-class capture for agents launched from a Linux desktop entry

- **Class:** OWNER (open; not ADOPT, ADAPT or REJECT yet)
- **Upstream permalinks:** [src/login-shell-environment.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/login-shell-environment.ts)
- **Study:** [desktop study](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Section 14 (row P-38 and the conflict note)
- **Reason:** CONFLICT, stays open: capturing the login-shell environment merges ambient variables over the inherited environment, which conflicts with environment scrubbing (A-58, Section 9 of the desktop study and the Env scrubbing lesson in the plugin study) and with the T07 and T13 environment rules of package 002. Only a PATH-class allowlist could be considered, and only after an owner decision.
- **Reuse statement:** Pattern only, reimplement. No upstream code or text is copied or imported (AGENTS.md section 7; upstream is Electron and TypeScript, LiteCowork is Tauri, Rust and React). No reuse is allowed before the owner decision.
- **LiteCowork tasks:** T07 and T13 of package 002; package 005 Environment admission
- **Where proposed:** PROP-P38 in `specs/` package 002 TASKS.md (PROPOSED, not accepted)
- **Build, test and deferral:** Blocked on the owner decision. Then a Linux desktop-entry launch test.

### P-39 Release approval bound to a digest of manifest plus plan; allowlisted release configuration

- **Class:** ADAPT
- **Upstream permalinks:** [scripts/publish-desktop-ci.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/publish-desktop-ci.ts), [scripts/desktop-package-environment.mjs](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/desktop-package-environment.mjs), [scripts/desktop-publish-route.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/desktop-publish-route.ts), [scripts/resolve-retained-desktop-publish-ci.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/resolve-retained-desktop-publish-ci.ts)
- **Study:** [desktop study](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Sections 11 and 14 (row P-39)
- **Reason:** It is the release-process twin of AGENTS.md section 6, "approval is bound to the plan digest". The CI shapes differ (GitHub Actions, no COS), so the digest-recompute rule and the ambient-variable filter are taken, not the scripts.
- **Reuse statement:** Pattern only, reimplement. No upstream code or text is copied or imported (AGENTS.md section 7; upstream is Electron and TypeScript, LiteCowork is Tauri, Rust and React).
- **LiteCowork tasks:** T09, T10, T15 of package 019
- **Where proposed:** PROP-P39 in `specs/` package 019 TASKS.md (PROPOSED, not accepted)
- **Build, test and deferral:** DEFERRED with B-43.

### P-40 Signed-file completion protocol (sign once, timestamp on fresh copies, compare normalized bytes, same-volume rename, never retry signing)

- **Class:** ADAPT
- **Upstream permalinks:** [scripts/windows-timestamp.mjs](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/windows-timestamp.mjs), [scripts/windows-sign.mjs](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/windows-sign.mjs), [scripts/windows-signing-state.mjs](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/windows-signing-state.mjs)
- **Study:** [desktop study](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Sections 11 (11.1) and 14 (row P-40)
- **Reason:** Independent of EV tokens; a persistent attempt interlock is only needed if LiteCowork uses a hardware token. The scripts depend on SignTool and Node, so they are not reusable as files.
- **Reuse statement:** Pattern only, reimplement. No upstream code or text is copied or imported (AGENTS.md section 7; upstream is Electron and TypeScript, LiteCowork is Tauri, Rust and React).
- **LiteCowork tasks:** T09 of package 019
- **Where proposed:** PROP-P40 in `specs/` package 019 TASKS.md (PROPOSED, not accepted)
- **Build, test and deferral:** DEFERRED; needs the Windows laptop and a signing identity (B-43).

### P-41 Installer and uninstaller safety matrix (data-home overlap, known-folder blocklist, handle pinning, no reparse traversal, honest progress)

- **Class:** ADAPT (as test cases only)
- **Upstream permalinks:** [installer/uninstall-data.h](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/installer/uninstall-data.h), [installer/uninstall.nsh](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/installer/uninstall.nsh), [installer/path.nsh](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/installer/path.nsh), [installer/extract.h](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/installer/extract.h), [installer/progress.h](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/installer/progress.h), [tests/expected/windows-uninstall.json](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/tests/expected/windows-uninstall.json), [tests/expected/windows-installer.json](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/tests/expected/windows-installer.json)
- **Study:** [desktop study](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Sections 11 (11.7) and 14 (row P-41)
- **Reason:** The five uninstall cases (interactive, silent, upgrade, replaced, home-inside) are a ready test outline. The C++ and NSIS code is not reused and the native GDI+ renderer is REJECT.
- **Reuse statement:** Pattern only, reimplement. No upstream code or text is copied or imported (AGENTS.md section 7; upstream is Electron and TypeScript, LiteCowork is Tauri, Rust and React). If the case list is copied verbatim: if any text or structure is carried over, MIT applies (upstream LICENSE, "Copyright (c) 2026 DeepSeek"): keep that copyright line and the MIT permission notice in the third-party notices produced by task T13 of package 019, and cite the upstream path and SHA `fcc0ad46` in the code comment.
- **LiteCowork tasks:** T08 of package 019; T04 of package 017
- **Where proposed:** PROP-P41 in `specs/` package 019 TASKS.md (PROPOSED, not accepted)
- **Build, test and deferral:** DEFERRED; Windows laptop.

### P-42 Journal-derived installed-update evidence (strict per-process journal, milestone inspector that prints what it cannot prove)

- **Class:** ADAPT
- **Upstream permalinks:** [src/update-journal.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/update-journal.ts), [scripts/installed-update-qualification.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/installed-update-qualification.ts), [scripts/installed-update-publication.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/installed-update-publication.ts)
- **Study:** [desktop study](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Sections 8, 11 (11.5) and 14 (row P-42)
- **Reason:** This is the concrete form of "reported, observed, committed, verified are different". The throwaway-feed publication step needs a distribution endpoint, which LiteCowork has not chosen.
- **Reuse statement:** Pattern only, reimplement. No upstream code or text is copied or imported (AGENTS.md section 7; upstream is Electron and TypeScript, LiteCowork is Tauri, Rust and React).
- **LiteCowork tasks:** T14, T15 and T19 of package 019
- **Where proposed:** PROP-P42 in `specs/` package 019 TASKS.md (PROPOSED, not accepted)
- **Build, test and deferral:** DEFERRED; Windows laptop first, then Linux.

### P-43 Lifecycle-script allowlist and classified patch policy for dependencies

- **Class:** ADAPT
- **Upstream permalinks:** [scripts/runtime-patch-policy.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/runtime-patch-policy.ts), [src/project-manager.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/project-manager.ts)
- **Study:** [desktop study](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Sections 11 and 14 (row P-43)
- **Reason:** Applies to LiteSPM-linked dependency handling (package 013, DEFERRED) and the release dependency audit (T13 of package 019). pnpm specifics do not carry over to cargo and the LiteSPM bridge.
- **Reuse statement:** Pattern only, reimplement. No upstream code or text is copied or imported (AGENTS.md section 7; upstream is Electron and TypeScript, LiteCowork is Tauri, Rust and React).
- **LiteCowork tasks:** Package 013 (DEFERRED); T13 of package 019
- **Where proposed:** PROP-P43 in `specs/` package 019 TASKS.md (PROPOSED, not accepted)
- **Build, test and deferral:** DEFERRED with LiteSPM (package 013).

### P-44 Streaming secret redactor for subprocess output that holds back a buffer whose prefix could still become a secret

- **Class:** ADOPT (algorithm port)
- **Upstream permalinks:** [scripts/packaging-run.mjs](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/packaging-run.mjs), [scripts/packaging-step.mjs](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/packaging-step.mjs)
- **Study:** [desktop study](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Sections 8 and 14 (row P-44)
- **Reason:** Whole-chunk replacement leaks a secret split across chunks. `packagingOutputRedactor` is a small pure algorithm, so it ports to Rust directly. It sits beside the masking in `scripts/v1_secret_scan.py` and the secret canary rule (T03 of package 017).
- **Reuse statement:** Algorithm only, reimplement in Rust (or in the LiteCowork TypeScript tree for UI logic); the upstream source has only Node imports so the algorithm ports directly, but the file itself is not imported. If any text or structure is carried over, MIT applies (upstream LICENSE, "Copyright (c) 2026 DeepSeek"): keep that copyright line and the MIT permission notice in the third-party notices produced by task T13 of package 019, and cite the upstream path and SHA `fcc0ad46` in the code comment.
- **LiteCowork tasks:** T07 of package 018; T03 of package 017
- **Where proposed:** PROP-P44 in `specs/` package 018 TASKS.md (PROPOSED, not accepted)
- **Build, test and deferral:** Not started. Rust unit tests with a secret split at every byte offset run on Linux.

### P-45 Default-deny webview and session permissions, including the main one, plus a guest network filter

- **Class:** ADAPT
- **Upstream permalinks:** [src/browser-guests.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/browser-guests.ts), [src/microphone-permissions.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/microphone-permissions.ts), [src/policy-test-auth.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/policy-test-auth.ts)
- **Study:** [desktop study](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Sections 9 and 14 (row P-45)
- **Reason:** Upstream shows a counter-example (the shell main session grants non-media permissions). Tauri has no equivalent default grant; keep it that way and add a test that notification, clipboard-read and geolocation requests are denied. The CSP audit already exists in `scripts/v1_csp_audit.py`.
- **Reuse statement:** Pattern only, reimplement. No upstream code or text is copied or imported (AGENTS.md section 7; upstream is Electron and TypeScript, LiteCowork is Tauri, Rust and React).
- **LiteCowork tasks:** T06 of package 019; T04 and T06 of package 008
- **Where proposed:** PROP-P45 in `specs/` package 019 TASKS.md (PROPOSED, not accepted)
- **Build, test and deferral:** Needs a packaged webview run on the Windows laptop and native Linux; DEFERRED with B-43.

### P-46 IME-safe shortcut dispatch rules (no match during composition, dead keys or AltGr; reset on blur; balanced keyups)

- **Class:** ADAPT (rules only)
- **Upstream permalinks:** [src/keyboard.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/keyboard.ts)
- **Study:** [desktop study](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Section 14 (row P-46)
- **Reason:** `src/keyboard.ts` imports `electron` and four workspace packages, so it is not reusable as a file; the rules become acceptance cases for the existing `src/shell/shortcut-registry.ts`.
- **Reuse statement:** Pattern only, reimplement. No upstream code or text is copied or imported (AGENTS.md section 7; upstream is Electron and TypeScript, LiteCowork is Tauri, Rust and React).
- **LiteCowork tasks:** T11 of package 011
- **Where proposed:** PROP-P46 in `specs/` package 011 TASKS.md (PROPOSED, not accepted)
- **Build, test and deferral:** UI tests on Linux; not started.

Reinforced earlier proposals: P-20 capability seam and P-27 durable run record plus process-group liveness fence.

## Plugin architecture (P-27 and related)

Paths are full permalinks at the pinned SHA. Study: [plugin study](DEEPSEEK-HARNESS-PLUGIN-ARCHITECTURE-DEEP-DIVE-2026-11.md) Section 8 (mapping) with the sections noted per row. All rows are PROPOSED, not accepted. Reuse statement for every row marked ADAPT or REJECT: pattern only, reimplement, nothing imported. Rows marked ADOPT are small rules or algorithms: reimplement; If any text or structure is carried over, MIT applies (upstream LICENSE, "Copyright (c) 2026 DeepSeek"): keep that copyright line and the MIT permission notice in the third-party notices produced by task T13 of package 019, and cite the upstream path and SHA `fcc0ad46` in the code comment.

| Pattern | Upstream permalink | Study section | Class | Reason | LiteCowork task(s) |
|---|---|---|---|---|---|
| Durable run record plus process-group liveness fence | [operations.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/packages/boot/plugin-manager/src/operations.ts) (`activeRecordedRun`), [run-tree.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/packages/boot/plugin-manager/src/run-tree.ts) | 2, 3, 8 | ADAPT (liveness check is an ADOPT candidate) | Crash-safe recovery after a restart | T08 of package 001 (restart sweep, done), package 005, package 013 (DEFERRED); PROP-P27 |
| Atomic restore of manifest and lock on rejected mutation | [operations.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/packages/boot/plugin-manager/src/operations.ts) | 2, 8 | ADAPT | Mirrors byte-exact restore in LiteSPM | Package 013 (DEFERRED), package 005 |
| Pre-flight compatibility read, keep the working version | [operations.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/packages/boot/plugin-manager/src/operations.ts) (`namedSpecManifest`), [versions.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/packages/boot/plugin-manager/src/versions.ts) | 2, 8 | ADAPT | Upgrade Shield (P-21) needs the incoming facts before replacing a working version | Package 002, package 013 |
| Credential refusal in approvals and install specs | [build-approval.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/packages/boot/plugin-manager/src/build-approval.ts), [install-spec.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/packages/boot/plugin-manager/src/install-spec.ts) (`HTTP_USER_INFO`) | 4, 5, 8 | ADOPT (small rules) | A few lines of rule; tokens never in config or prompts | Package 013 (DEFERRED), package 006 |
| Absolute-path-only install specs | [install-spec.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/packages/boot/plugin-manager/src/install-spec.ts) (`parseInstallSpec`) | 5, 8 | ADOPT (rule) | Never resolve against an implicit working directory | Package 013 (DEFERRED) |
| Fail-closed reachability probe before network operations | [git-transport.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/packages/boot/plugin-manager/src/git-transport.ts) | 5, 8 | ADAPT | Probe before a network write | Package 006, package 013 |
| Env scrubbing by execution mode | [operations.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/packages/boot/plugin-manager/src/operations.ts) (`scrubbedParentEnv`) | 2 | ADAPT | Basis of the conflict with P-38 | Package 002 (P-38 stays OPEN) |
| Capability seam (Definition, Provider, Consumer) | [docs/capability-seams.md](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/docs/capability-seams.md) | 6, 8 | ADAPT (ADR P-20, proposed) | A clean seam between Environments, terminal, browser and agents | Packages 002, 005, 008 |
| In-process plugins, lazy client plugin loading | [client-plugin-loading-model.md](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/.agents/notes/implemented/architecture/2026-07-23-client-plugin-loading-model.md) | 7, 8 | REJECT for V1 | Package 016 forbids model HTML; in-process plugins are not LiteCowork architecture | None |

## Rejected (do not re-propose)

Class REJECT for all rows. Reuse statement: none, nothing is carried over. Study: [desktop study](DEEPSEEK-HARNESS-DESKTOP-DEEP-DIVE-2026-11.md) Section 14 ("Reject") unless noted.

| Rejected item | Upstream permalink | Reason |
|---|---|---|
| Loopback HTTP Host with cookie and custom-scheme proxy | [src/host-process.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/host-process.ts) | LiteCowork uses UDS and named pipe (Section 2 of the study) |
| In-process plugins | [client-plugin-loading-model.md](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/.agents/notes/implemented/architecture/2026-07-23-client-plugin-loading-model.md) | Not LiteCowork architecture; package 016 forbids model HTML |
| `runAsNode` | [src/host-process.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/host-process.ts) | Electron specific |
| Mandatory-update policy service | [src/mandatory-update-policy.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/mandatory-update-policy.ts) | Product policy not wanted; no forced updates in V1 |
| Analytics and Platform accounts | [src/main.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/main.ts) | Out of V1 scope |
| macOS vibrancy and notarization | [electron-builder.config.mjs](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/electron-builder.config.mjs) | macOS is not V1 |
| Native GDI+ installer renderer | [installer/](https://github.com/deepseek-ai/deepseek-harness/tree/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/installer) | Only the test cases of P-41 are taken |
| COS and EdgeOne publishing | [scripts/desktop-cos.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/desktop-cos.ts) | Vendor specific; P-36 takes the discipline only |
| Bundled Python, Node and pnpm runtimes for V1 | [scripts/runtime-patch-policy.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/runtime-patch-policy.ts) | LiteCowork ships a Rust daemon |
| `dsh` PATH command install | [src/command-installation.ts](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/src/command-installation.ts) | Reference only if a CLI shim is ever added |
| Wine-based NSIS bootstrap signing patch | [scripts/windows-sign.mjs](https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/apps/desktop/scripts/windows-sign.mjs) | Not applicable; the signing environment is the Windows laptop |

## Status of adoption

| State | Items |
|---|---|
| Already reflected in code (independent implementation, same principle) | 011-T13 prerequisite read-only stop preview with UNKNOWN as busy; 019-T12 updater state reducer; 019-T14 row-aware matrix; 019-T15 evidence bundle with secret scan; 011-T04 IPC admission bounds; 011-T11 shortcut registry |
| Open proposals needing owner review before any package change | P-31 to P-46 (A-63), each recorded as a PROPOSED `PROP-Pnn` row in the owning package TASKS.md, PLAN.md and RESEARCH.md (not accepted); P-38 conflicts with environment scrubbing (A-58) and stays OPEN |
| Deferred with their packages | LiteSPM-linked (P-43), release and signing (P-35, P-36, P-39, P-40, P-42), Windows (P-41) |

When an owner accepts a proposal: add a task line in the owning package TASKS.md that names the upstream path and mode from this table, then update this map's adoption state.
