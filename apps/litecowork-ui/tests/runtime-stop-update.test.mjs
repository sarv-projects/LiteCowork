import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { after, test } from "node:test";
import { fileURLToPath } from "node:url";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import react from "@vitejs/plugin-react";
import { createServer } from "vite";
import {
  STOP_COMMAND_AVAILABLE,
  runtimeStoppedLabel,
  summarizeStopPreview,
} from "../src/runtime/runtime-stop-state.ts";

const vite = await createServer({
  configFile: false,
  root: fileURLToPath(new URL("..", import.meta.url)),
  plugins: [react()],
  server: { middlewareMode: true, hmr: false },
  optimizeDeps: { noDiscovery: true, include: [] },
  appType: "custom",
});
const { RuntimeStopPreviewPanel } = await vite.ssrLoadModule("/src/runtime/RuntimeStopPreviewPanel.tsx");
after(async () => vite.close());

const expected = { runtimeId: "rt", incarnationId: "inc" };

function preview(overrides = {}) {
  return {
    status: "READY",
    preview: {
      runtimeId: "rt",
      runtimeIncarnationId: "inc",
      observedAt: "2026-11-10T00:00:00Z",
      admittedTurnIds: [],
      admittedAttemptIds: [],
      unresolvedEffectIds: [],
      executionLeaseIds: [],
      environmentControlLeaseIds: [],
      providerReferenceIds: [],
      pendingCleanupIds: [],
      verdict: "UNKNOWN",
      blockers: [{ blockerId: "b1", code: "RUNTIME_QUIESCENCE_UNKNOWN", safeMessage: "Live sessions cannot be observed yet.", resolutionHint: "Wait." }],
      ...overrides,
    },
  };
}

test("Stop safely is never enabled while the stop command is unmounted, even on QUIESCENT", () => {
  assert.equal(STOP_COMMAND_AVAILABLE, false);
  const summary = summarizeStopPreview(preview({ verdict: "QUIESCENT", blockers: [] }), expected);
  assert.equal(summary.stopEnabled, false);
  assert.match(summary.stopReason, /unavailable until the authenticated stop command exists/);
});

test("BLOCKED lists what is running and refuses stop and update (AC-014, AC-015)", () => {
  const summary = summarizeStopPreview(
    preview({ verdict: "BLOCKED", admittedTurnIds: ["t1", "t2"], unresolvedEffectIds: ["e1"] }),
    expected,
  );
  assert.equal(summary.stopEnabled, false);
  assert.equal(summary.updateGate.allowed, false);
  assert.match(summary.updateGate.message, /work is still running/);
  assert.deepEqual(summary.items, [
    { label: "Active Conversation turns", count: 2 },
    { label: "Unreconciled external Effects", count: 1 },
  ]);
});

test("UNKNOWN refuses update with the reason and never reads as nothing running", () => {
  const summary = summarizeStopPreview(preview(), expected);
  assert.equal(summary.updateGate.allowed, false);
  assert.match(summary.verdictLabel, /Cannot confirm/);
  assert.deepEqual(summary.reasons, ["Live sessions cannot be observed yet."]);
});

test("only a positive QUIESCENT verdict for the exact incarnation opens the update gate", () => {
  const ok = summarizeStopPreview(preview({ verdict: "QUIESCENT", blockers: [] }), expected);
  assert.equal(ok.updateGate.allowed, true);
  const stale = summarizeStopPreview(preview({ verdict: "QUIESCENT", blockers: [] }), { runtimeId: "rt", incarnationId: "other" });
  assert.equal(stale.updateGate.allowed, false);
  assert.match(stale.updateGate.message, /different Runtime incarnation/);
  for (const state of [{ status: "IDLE" }, { status: "LOADING" }, { status: "ERROR", message: "no bridge" }]) {
    assert.equal(summarizeStopPreview(state, expected).updateGate.allowed, false, state.status);
  }
});

test("window close never renders Runtime stopped; only an observed STOPPED recovery state does", () => {
  assert.equal(runtimeStoppedLabel("READY"), null);
  assert.equal(runtimeStoppedLabel("DRAINING"), null);
  assert.equal(runtimeStoppedLabel(null), null);
  assert.equal(runtimeStoppedLabel("STOPPED"), "Runtime stopped");
});

test("panel renders the verdict, reasons, disabled Stop safely text and refused gate", () => {
  const html = renderToStaticMarkup(createElement(RuntimeStopPreviewPanel, { state: preview(), runtimeId: "rt", incarnationId: "inc" }));
  assert.match(html, /data-stop-verdict="UNKNOWN"/);
  assert.match(html, /Live sessions cannot be observed yet\./);
  assert.match(html, /data-update-gate="refused"/);
  assert.match(html, /data-stop-safely="unavailable"/);
  assert.doesNotMatch(html, /<button/);
});

test("Settings renders Stop safely as text and the preview control is mapped", async () => {
  const source = await readFile(new URL("../src/settings/SettingsView.tsx", import.meta.url), "utf8");
  assert.match(source, /data-control-id="runtime-stop-preview"/);
  assert.doesNotMatch(source, /data-control-id="runtime-stop-safely"/);
  assert.doesNotMatch(source, /Runtime stopped/);
});
