import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const app = await readFile(new URL("../src/App.tsx", import.meta.url), "utf8");

function pageUnion() {
  const match = app.match(/type Page = ([^;]+);/);
  assert.ok(match, "Page union present");
  return [...match[1].matchAll(/"([^"]+)"/g)].map(item => item[1]);
}

function navigationLabels() {
  const block = app.match(/export const navigation[\s\S]*?\n\];/);
  assert.ok(block, "navigation array present");
  return [...block[0].matchAll(/label: "([^"]+)"/g)].map(item => item[1]);
}

test("every mounted Page has a navigation path (AC-021)", () => {
  const labels = new Set(navigationLabels());
  for (const page of pageUnion()) {
    assert.ok(labels.has(page), `${page} is in the Page union but has no navigation entry`);
  }
});

test("navigation has no label outside the Page union", () => {
  const pages = new Set(pageUnion());
  for (const label of navigationLabels()) assert.ok(pages.has(label), `${label} is not a Page`);
});

test("Back and Forward are removed because navigation never pushes history (AC-021)", () => {
  assert.doesNotMatch(app, /window\.history\.(back|forward)\(/);
  assert.doesNotMatch(app, /aria-label="(Back|Forward)"/);
  assert.doesNotMatch(app, /history\.pushState|history\.replaceState/);
});

function sidebarSource() {
  const start = app.indexOf('id="main-navigation"');
  const end = app.indexOf("</aside>", start);
  assert.ok(start > 0 && end > start, "sidebar block present");
  return app.slice(start, end);
}

test("every Page is reachable from the rendered sidebar or a documented in-page link (AC-021)", () => {
  const sidebar = sidebarSource();
  const direct = new Set([...sidebar.matchAll(/requestPageNavigation\("([^"]+)"\)/g)].map(item => item[1]));
  // Conversations is the landing page and the New Chat target.
  direct.add("Conversations");
  const more = app.match(/const SIDEBAR_MORE_PAGES: Page\[\] = \[([^\]]*)\]/);
  assert.ok(more, "SIDEBAR_MORE_PAGES present");
  for (const match of more[1].matchAll(/"([^"]+)"/g)) direct.add(match[1]);
  assert.match(sidebar, /navigation\.filter\(\(item\) => SIDEBAR_MORE_PAGES\.includes\(item\.label\)\)\.map/);
  // Routines is reached from the Automations page (requestPageNavigation("Routines") outside the sidebar).
  const inPage = new Set(["Routines"]);
  assert.match(app, /requestPageNavigation\("Routines"\)/);
  for (const page of pageUnion()) {
    assert.ok(direct.has(page) || inPage.has(page), `${page} is not reachable from the sidebar`);
  }
});
