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
