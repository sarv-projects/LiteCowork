import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { readdir } from "node:fs/promises";
import test from "node:test";
import { SETTINGS_CONTROL_MAP } from "../src/settings/settings-control-map.ts";

const app = await readFile(new URL("../src/App.tsx", import.meta.url), "utf8");
const view = await readFile(new URL("../src/settings/SettingsView.tsx", import.meta.url), "utf8");

test("one Settings authority remains: SettingsPage and its helpers are deleted (AC-023)", async () => {
  assert.doesNotMatch(app, /function SettingsPage\b/);
  assert.doesNotMatch(app, /SettingsPageProps/);
  assert.doesNotMatch(app, /function localServiceCopy/);
  const sources = [];
  async function walk(directory) {
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      const path = new URL(`${entry.name}${entry.isDirectory() ? "/" : ""}`, directory);
      if (entry.isDirectory()) await walk(path);
      else if (/\.(tsx?|mjs)$/.test(entry.name)) sources.push(await readFile(path, "utf8"));
    }
  }
  await walk(new URL("../src/", import.meta.url));
  assert.equal(sources.filter(text => /export function Settings(Page|View)\b/.test(text)).length, 1, "exactly one Settings surface");
  assert.equal(sources.filter(text => /<SettingsPage\b/.test(text)).length, 0);
});

test("merged features are reachable and mapped: conflict recovery, enrollment, history", () => {
  for (const id of ["workspaces-instructions-use-latest", "runtime-enroll", "runtime-enrollment-refresh"]) {
    assert.match(view, new RegExp(`data-control-id="${id}"`));
    assert.ok(SETTINGS_CONTROL_MAP[id], `${id} is mapped`);
  }
  assert.match(view, /Use this version/);
  assert.match(view, /Revision history and source provenance/);
  assert.match(view, /This computer’s Workspace access/);
});

test("each setting with a Coworker or Automation override carries a precedence note (AC-023)", () => {
  for (const note of ["workspace-instructions", "default-agent"]) {
    assert.match(view, new RegExp(`data-precedence-note="${note}"`), note);
  }
});

test("there is one navigation array and the sidebar renders from it", () => {
  assert.equal((app.match(/export const navigation\b/g) ?? []).length, 1);
  assert.match(app, /navigation\.filter\(/);
  assert.doesNotMatch(app, /const settingsNavigation|const navigationItems/);
});
