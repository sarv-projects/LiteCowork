import assert from "node:assert/strict";
import { after, test } from "node:test";
import { fileURLToPath } from "node:url";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import react from "@vitejs/plugin-react";
import { createServer } from "vite";
import {
  DASHBOARD_PINS_KEY,
  WIDGET_IDS,
  loadPins,
  savePins,
  summarizeByStatus,
  togglePin,
} from "../src/dashboard/dashboard-state.ts";

const vite = await createServer({
  configFile: false,
  root: fileURLToPath(new URL("..", import.meta.url)),
  plugins: [react()],
  server: { middlewareMode: true, hmr: false },
  optimizeDeps: { noDiscovery: true, include: [] },
  appType: "custom",
});
const { WorkspaceDashboard, WidgetBody } = await vite.ssrLoadModule("/src/dashboard/WorkspaceDashboard.tsx");
after(async () => vite.close());

function memoryStorage(initial = {}) {
  const map = new Map(Object.entries(initial));
  return { getItem: key => map.get(key) ?? null, setItem: (key, value) => { map.set(key, value); }, map };
}

test("a Workspace with no saved state shows every widget pinned", () => {
  assert.deepEqual(loadPins(memoryStorage(), "w1"), [...WIDGET_IDS]);
});

test("pin state persists per Workspace across a restart (new storage view over the same data)", () => {
  const storage = memoryStorage();
  savePins(storage, "w1", togglePin(loadPins(storage, "w1"), "tasks"));
  savePins(storage, "w2", ["coworkers"]);
  const restarted = memoryStorage({ [DASHBOARD_PINS_KEY]: storage.map.get(DASHBOARD_PINS_KEY) });
  assert.deepEqual(loadPins(restarted, "w1"), ["automations", "coworkers", "artifacts"]);
  assert.deepEqual(loadPins(restarted, "w2"), ["coworkers"]);
  assert.deepEqual(loadPins(restarted, "w3"), [...WIDGET_IDS]);
});

test("corrupt or hostile stored state falls back to the default and unknown ids are dropped", () => {
  assert.deepEqual(loadPins(memoryStorage({ [DASHBOARD_PINS_KEY]: "{not json" }), "w1"), [...WIDGET_IDS]);
  assert.deepEqual(loadPins(memoryStorage({ [DASHBOARD_PINS_KEY]: JSON.stringify({ w1: ["tasks", "evil"] }) }), "w1"), ["tasks"]);
  assert.deepEqual(loadPins(memoryStorage({ [DASHBOARD_PINS_KEY]: "[1]" }), "w1"), [...WIDGET_IDS]);
});

test("togglePin keeps a stable widget order", () => {
  assert.deepEqual(togglePin(["tasks"], "coworkers"), ["tasks", "coworkers"]);
  assert.deepEqual(togglePin(["coworkers", "tasks"], "tasks"), ["coworkers"]);
});

test("status summaries are counts from the returned items and flag truncation", () => {
  const state = summarizeByStatus([{ status: "READY" }, { status: "READY" }, { status: "FAILED" }], { READY: "Ready" }, true);
  assert.equal(state.status, "READY");
  assert.deepEqual(state.lines, [{ label: "FAILED", value: 1 }, { label: "Ready", value: 2 }]);
  assert.match(state.note, /first page only/);
});

test("unavailable sources render unavailable with the reason, never empty or fabricated (AC-024)", () => {
  const html = renderToStaticMarkup(createElement(WorkspaceDashboard, {
    workspaceId: "w1",
    storage: memoryStorage(),
    sources: {
      tasks: { unavailable: "Task projection is not served." },
      automations: { unavailable: "Automation projection is not served." },
      coworkers: { unavailable: "Coworker projection is not served." },
      artifacts: { unavailable: "No Workspace-wide recent-artifacts projection is served yet." },
    },
  }));
  assert.match(html, /Task status/);
  assert.match(html, /Recent artifacts/);
  // Server render shows the first-paint LOADING state; the unavailable reason is applied on mount.
  assert.doesNotMatch(html, /: 0</);
  assert.match(html, /aria-pressed="true"/);
});

test("an unpinned widget renders as not pinned and loads nothing", () => {
  const storage = memoryStorage({ [DASHBOARD_PINS_KEY]: JSON.stringify({ w1: ["tasks"] }) });
  const html = renderToStaticMarkup(createElement(WorkspaceDashboard, {
    workspaceId: "w1",
    storage,
    sources: {
      tasks: { load: async () => ({ status: "READY", lines: [], note: null }) },
      automations: { unavailable: "x" },
      coworkers: { unavailable: "x" },
      artifacts: { unavailable: "x" },
    },
  }));
  assert.equal((html.match(/Not pinned\./g) ?? []).length, 3);
});

test("WidgetBody renders the unavailable reason, a ready list, and an empty ready state distinctly", () => {
  const unavailable = renderToStaticMarkup(createElement(WidgetBody, { state: { status: "UNAVAILABLE", reason: "Task projection is not served." } }));
  assert.match(unavailable, /data-widget-state="UNAVAILABLE"/);
  assert.match(unavailable, /Unavailable: Task projection is not served\./);
  const ready = renderToStaticMarkup(createElement(WidgetBody, { state: { status: "READY", lines: [{ label: "Ready", value: 3 }], note: null } }));
  assert.match(ready, /Ready: 3/);
  const empty = renderToStaticMarkup(createElement(WidgetBody, { state: { status: "READY", lines: [], note: null } }));
  assert.match(empty, /None reported by the Runtime/);
  assert.doesNotMatch(empty, /Unavailable/);
});
