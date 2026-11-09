import test from "node:test";
import assert from "node:assert/strict";
import {
  UpdateCommandError,
  createUpdateController,
  describeUpdateState,
  initialUpdateState,
  installAvailability,
  updateReducer,
  type UpdateAction,
  type UpdateApi,
  type UpdateState,
} from "../src/updates/update-state.ts";
import { parseCheckOutcome, parseInstallOutcome, toUpdateCommandError } from "../src/updates/desktop-update-api.ts";

const windows = { os: "windows" as const, signed: true };

function run(actions: UpdateAction[], from: UpdateState = initialUpdateState(windows)) {
  return actions.reduce(updateReducer, from);
}

test("initial state reflects the row and signature", () => {
  assert.equal(initialUpdateState(windows).status, "IDLE");
  assert.equal(initialUpdateState({ os: "windows", signed: false }).status, "UNSIGNED_DEV");
  assert.equal(initialUpdateState({ os: "linux", signed: true }).status, "LINUX_MANUAL");
  assert.match(describeUpdateState({ status: "LINUX_MANUAL" }), /installing the new package/);
});

test("up to date is shown only after a verified UpToDate outcome", () => {
  const checking = run([{ type: "CHECK_REQUESTED" }]);
  assert.equal(checking.status, "CHECKING");
  const failed = updateReducer(checking, { type: "CHECK_ERROR", code: "ManifestUnreachable", offline: false });
  assert.equal(failed.status, "CHECK_FAILED");
  assert.doesNotMatch(describeUpdateState(failed), /up to date/i);
  const offline = updateReducer(checking, { type: "CHECK_ERROR", code: "ManifestUnreachable", offline: true });
  assert.equal(offline.status, "OFFLINE");
  assert.doesNotMatch(describeUpdateState(offline), /up to date/i);
  assert.match(describeUpdateState(updateReducer(checking, { type: "CHECK_RESULT", outcome: { kind: "UpToDate", current: "0.1.0" } })), /up to date/i);
});

test("a stray result without a request cannot create UP_TO_DATE", () => {
  const idle = initialUpdateState(windows);
  assert.equal(updateReducer(idle, { type: "CHECK_RESULT", outcome: { kind: "UpToDate", current: "1" } }).status, "IDLE");
});

test("install is disabled with a reason outside an offer", () => {
  for (const status of ["IDLE", "CHECKING", "UP_TO_DATE", "OFFLINE", "HANDOFF"] as const) {
    const state = status === "UP_TO_DATE" ? { status, current: "1" }
      : status === "HANDOFF" ? { status, version: "2" } : { status };
    const availability = installAvailability(state as UpdateState);
    assert.equal(availability.enabled, false, status);
    assert.ok(!availability.enabled && availability.reason.length > 10);
  }
});

const available: UpdateState = { status: "AVAILABLE", version: "0.2.0", notesDigest: "d", channel: "stable" };

test("double activation of install enters GATING once and keeps one request id", () => {
  const once = updateReducer(available, { type: "INSTALL_REQUESTED", requestId: "r1" });
  const twice = updateReducer(once, { type: "INSTALL_REQUESTED", requestId: "r2" });
  assert.deepEqual(twice, once);
  assert.equal(once.status === "GATING" && once.requestId, "r1");
});

test("refusal lists reasons and offers retry with a new request id", () => {
  const gating = updateReducer(available, { type: "INSTALL_REQUESTED", requestId: "r1" });
  const refused = updateReducer(gating, {
    type: "INSTALL_RESULT",
    outcome: { kind: "Refused", reasons: [{ label: "Task A", status: "running" }, { label: "Effect B", status: "unknown" }] },
  });
  assert.equal(refused.status, "REFUSED_NOT_QUIESCENT");
  assert.equal(installAvailability(refused).enabled, true);
  const retry = updateReducer(refused, { type: "INSTALL_REQUESTED", requestId: "r2" });
  assert.equal(retry.status === "GATING" && retry.requestId, "r2");
});

test("an unavailable gate refuses rather than installs", () => {
  const gating = updateReducer(available, { type: "INSTALL_REQUESTED", requestId: "r1" });
  const refused = updateReducer(gating, { type: "INSTALL_ERROR", code: "QuiescenceUnavailable" });
  assert.equal(refused.status, "REFUSED_NOT_QUIESCENT");
  assert.match(describeUpdateState(refused), /could not confirm/);
});

test("handoff and typed failures render specific text", () => {
  const gating = updateReducer(available, { type: "INSTALL_REQUESTED", requestId: "r1" });
  assert.equal(updateReducer(gating, { type: "INSTALL_RESULT", outcome: { kind: "Handoff", version: "0.2.0" } }).status, "HANDOFF");
  const failed = updateReducer(gating, { type: "INSTALL_ERROR", code: "SignatureInvalid" });
  assert.match(describeUpdateState(failed), /signature/);
  const unique = new Set(["ManifestUnreachable", "SignatureInvalid", "Downgrade", "ChannelMismatch", "RuntimeStopFailed", "InstallerFailed"]
    .map((code) => describeUpdateState({ status: "FAILED", version: "1", code: code as never })));
  assert.equal(unique.size, 6);
});

test("unsigned and Linux rows ignore every action", () => {
  for (const platform of [{ os: "windows" as const, signed: false }, { os: "linux" as const, signed: true }]) {
    const state = initialUpdateState(platform);
    assert.equal(updateReducer(state, { type: "CHECK_REQUESTED" }), state);
  }
});

test("controller sends one install command for a double activation", async () => {
  let state: UpdateState = available;
  const sent: string[] = [];
  let release: (() => void) | undefined;
  const api: UpdateApi = {
    async checkForUpdate() { throw new Error("unused"); },
    installUpdate({ requestId }) {
      sent.push(requestId);
      return new Promise((resolve) => { release = () => resolve({ kind: "Handoff", version: "0.2.0" }); });
    },
  };
  let counter = 0;
  const controller = createUpdateController({
    api,
    dispatch: (action) => { state = updateReducer(state, action); },
    getState: () => state,
    newRequestId: () => `req-${++counter}`,
    isOffline: () => false,
  });
  const first = controller.install();
  const second = controller.install();
  release?.();
  await Promise.all([first, second]);
  assert.deepEqual(sent, ["req-1"]);
  assert.equal(state.status, "HANDOFF");
});

test("controller maps a failed check to CHECK_FAILED or OFFLINE", async () => {
  for (const offline of [false, true]) {
    let state: UpdateState = { status: "IDLE" };
    const controller = createUpdateController({
      api: {
        async checkForUpdate() { throw new UpdateCommandError("SignatureInvalid"); },
        async installUpdate() { throw new Error("unused"); },
      },
      dispatch: (action) => { state = updateReducer(state, action); },
      getState: () => state,
      newRequestId: () => "x",
      isOffline: () => offline,
    });
    await controller.check();
    assert.equal(state.status, offline ? "OFFLINE" : "CHECK_FAILED");
  }
});

test("wire parsing accepts typed outcomes and rejects unknown shapes", () => {
  assert.deepEqual(parseCheckOutcome({ kind: "UpToDate", current: "0.1.0" }), { kind: "UpToDate", current: "0.1.0" });
  assert.equal(parseCheckOutcome({ kind: "Available", version: "0.2.0", notes_digest: "d", channel: "stable" }).kind, "Available");
  assert.throws(() => parseCheckOutcome({ kind: "UpToDate" }), UpdateCommandError);
  assert.throws(() => parseCheckOutcome("up to date"), UpdateCommandError);
  assert.throws(() => parseInstallOutcome({ kind: "Nope" }), UpdateCommandError);
  const refused = parseInstallOutcome({ kind: "Refused", reasons: [{ label: "Task", status: "running" }, { label: "X", status: "weird" }] });
  assert.equal(refused.kind === "Refused" && refused.reasons[1]?.status, "unknown");
  assert.equal(toUpdateCommandError("Downgrade", "InstallerFailed").code, "Downgrade");
  assert.equal(toUpdateCommandError("command not found", "InstallerFailed").code, "InstallerFailed");
});
