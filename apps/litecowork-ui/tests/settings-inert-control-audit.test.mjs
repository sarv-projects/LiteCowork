import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { SETTINGS_CONTROL_MAP } from "../src/settings/settings-control-map.ts";

const source = await readFile(new URL("../src/settings/SettingsView.tsx", import.meta.url), "utf8");
const elements = [...source.matchAll(/<(button|input|select|textarea)\b([^>]*?)(?:\n|\s)/g)];

test("every interactive element in SettingsView declares a control id", () => {
  const tags = [...source.matchAll(/<(button|input|select|textarea)\b/g)];
  const ids = [...source.matchAll(/<(?:button|input|select|textarea) data-control-id="([^"]+)"/g)].map(match => match[1]);
  assert.ok(tags.length > 0);
  assert.equal(ids.length, tags.length, "an interactive element lacks data-control-id");
  assert.equal(new Set(ids).size, ids.length, "duplicate data-control-id");
  assert.ok(elements.length > 0);
});

test("every control id has a control-map entry with a served operation or applied local effect (AC-020)", () => {
  const ids = [...source.matchAll(/data-control-id="([^"]+)"/g)].map(match => match[1]);
  for (const id of ids) {
    const entry = SETTINGS_CONTROL_MAP[id];
    assert.ok(entry, `no control-map entry for ${id}`);
    assert.ok(entry.effect.length > 0, `empty effect for ${id}`);
  }
  for (const id of Object.keys(SETTINGS_CONTROL_MAP)) {
    assert.ok(ids.includes(id), `control-map entry ${id} is not rendered`);
  }
});

test("no Target-class control is interactive in the V1 map", () => {
  for (const [id, entry] of Object.entries(SETTINGS_CONTROL_MAP)) {
    assert.notEqual(entry.class, "Target", `${id} is target-only and must render as text`);
  }
});

test("shortcut labels stay removed until the keyboard registry exists", () => {
  assert.doesNotMatch(source, /<kbd/);
});

test("Usage, Storage and About render as unavailable text with a prerequisite, not controls", () => {
  for (const label of ["Usage &amp; Monitoring", "Storage &amp; Backup", "About &amp; Diagnostics"]) {
    const index = source.indexOf(label);
    assert.ok(index > 0, `${label} missing`);
    const item = source.slice(index, index + 200);
    assert.match(item, /Unavailable until/);
    assert.doesNotMatch(item, /<button|<input|<select/);
  }
});
