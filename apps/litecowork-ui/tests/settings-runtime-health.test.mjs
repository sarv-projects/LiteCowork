import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { deriveRuntimeHealthRows } from "../src/runtime/runtime-health.ts";

test("STOPPED, STARTING and RECOVERING never show execution open", () => {
  for (const recoveryState of ["STOPPED", "STARTING", "RECOVERING", "DRAINING", "STOPPING", null]) {
    for (const operatorReady of [true, false]) {
      const rows = deriveRuntimeHealthRows({ operatorReady, recoveryState }, "Linux x86_64");
      assert.equal(rows.executionReady, false, `${recoveryState}/${operatorReady}`);
      assert.match(rows.execution, /^Closed/);
    }
  }
});

test("DEGRADED while Operator is serving keeps commands available and execution closed", () => {
  const rows = deriveRuntimeHealthRows({ operatorReady: true, recoveryState: "DEGRADED" }, "Linux");
  assert.equal(rows.executionReady, false);
  assert.match(rows.operator, /^Serving/);
  assert.match(rows.execution, /Operator commands remain available/);
});

test("READY and serving is the only open execution state", () => {
  const rows = deriveRuntimeHealthRows({ operatorReady: true, recoveryState: "READY" }, "Linux");
  assert.equal(rows.executionReady, true);
  assert.equal(rows.execution, "Open");
  assert.equal(deriveRuntimeHealthRows({ operatorReady: false, recoveryState: "READY" }, "Linux").executionReady, false);
});

test("transport label names the real transport per platform", () => {
  assert.equal(deriveRuntimeHealthRows({ operatorReady: true, recoveryState: "READY" }, "Win32").transport, "Local IPC (named pipe)");
  assert.equal(deriveRuntimeHealthRows({ operatorReady: true, recoveryState: "READY" }, "Linux x86_64").transport, "Local IPC (socket)");
});

test("Settings no longer says Localhost and renders loading and error states in live regions", async () => {
  const source = await readFile(new URL("../src/settings/SettingsView.tsx", import.meta.url), "utf8");
  assert.doesNotMatch(source, /Localhost/);
  assert.match(source, /props\.runtimeLifecycleLoading/);
  assert.match(source, /props\.runtimeLifecycleError/);
  assert.match(source, /props\.runtimeError/);
  assert.match(source, /role="status"/);
  assert.match(source, /role="alert"/);
});
