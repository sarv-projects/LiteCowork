import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { AGENT_CATALOG_CONTROL_MAP } from "../src/settings/settings-control-map.ts";

const source = await readFile(new URL("../src/agents/AgentCatalogSettings.tsx", import.meta.url), "utf8");

test("every interactive element in AgentCatalogSettings declares a unique control id (011-T09)", () => {
  const tags = [...source.matchAll(/<(button|input|select|textarea)\b/g)];
  const ids = [...source.matchAll(/<(?:button|input|select|textarea) data-control-id="([^"]+)"/g)].map(match => match[1]);
  assert.ok(tags.length >= 30);
  assert.equal(ids.length, tags.length, "an interactive element lacks data-control-id");
  assert.equal(new Set(ids).size, ids.length, "duplicate data-control-id");
});

test("every rendered id has a map entry and every map entry is rendered", () => {
  const ids = [...source.matchAll(/data-control-id="([^"]+)"/g)].map(match => match[1]);
  for (const id of ids) {
    const entry = AGENT_CATALOG_CONTROL_MAP[id];
    assert.ok(entry, `no control-map entry for ${id}`);
    assert.ok(entry.effect.length > 0, `empty effect for ${id}`);
  }
  for (const id of Object.keys(AGENT_CATALOG_CONTROL_MAP)) {
    assert.ok(ids.includes(id), `map entry ${id} is not rendered`);
  }
});

test("no Target-class control is interactive and no control maps to an unserved operation", () => {
  for (const [id, entry] of Object.entries(AGENT_CATALOG_CONTROL_MAP)) {
    assert.notEqual(entry.class, "Target", `${id} is target-only and must render as text`);
    assert.doesNotMatch(entry.effect, /target-only|not served/i, id);
  }
});

test("descriptor, sign-in and install/update operations are text, not controls", () => {
  const ids = Object.keys(AGENT_CATALOG_CONTROL_MAP).join(" ");
  assert.doesNotMatch(ids, /sign-?in|install|update|login/i);
});
