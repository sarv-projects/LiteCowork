import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { agentSettingsDescription } from "../src/settings/agents-heading.ts";

test("heading names only harnesses with a probed profile", () => {
  const text = agentSettingsDescription([{ displayName: "OpenCode" }, { displayName: "OpenCode" }]);
  assert.match(text, /OpenCode/);
  assert.doesNotMatch(text, /Codex|Claude|Gemini/);
});

test("heading with no probes names no harness and says so", () => {
  const text = agentSettingsDescription([]);
  assert.match(text, /No agent harness has been probed/);
  assert.doesNotMatch(text, /OpenCode|Codex|Claude|Gemini/);
});

test("SettingsView no longer hard-codes a harness roster in the Agents heading", async () => {
  const view = await readFile(new URL("../src/settings/SettingsView.tsx", import.meta.url), "utf8");
  assert.doesNotMatch(view, /OpenCode, Codex, Claude Code, Gemini/);
  assert.match(view, /agentSettingsDescription\(props\.agentProfiles\)/);
});

test("unserved descriptor reads render as unavailable, not ready (AC-019)", async () => {
  const panel = await readFile(new URL("../src/agents/AgentControlDescriptorPanel.tsx", import.meta.url), "utf8");
  assert.match(panel, /DEPENDENCY_UNAVAILABLE/);
  assert.match(panel, /status: "UNAVAILABLE"/);
});
