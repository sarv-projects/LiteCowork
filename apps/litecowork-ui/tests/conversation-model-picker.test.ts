import assert from "node:assert/strict";
import test from "node:test";
import type { components } from "../src/api/operator-api.ts";
import {
  deriveConversationAgentSurface,
  type ConversationComposerState,
} from "../src/conversations/conversation-agent-surface.ts";
import {
  buildModelSelection,
  deriveCommandList,
  deriveModelPicker,
} from "../src/conversations/conversation-model-picker.ts";

type Descriptor = components["schemas"]["AgentControlDescriptor"];
const NOW = Date.parse("2026-10-18T12:01:00Z");

function descriptor(options: Descriptor["session_options"], commands = false): Descriptor {
  return {
    descriptor_digest: `sha256:${"a".repeat(64)}`,
    descriptor_api_version: 1,
    registry_agent_id: "agent-a",
    module_id: "agent-a-module",
    module_version: "1.0.0",
    installation: {
      registry_agent_id: "agent-a", module_id: "agent-a-module", runtime_id: "r", runtime_incarnation_id: "i",
      state: "INSTALLED", installed_version: "1.0.0", available_version: null, installation_source: null,
      observed_at: "2026-10-18T12:00:00Z", expires_at: "2026-10-18T12:05:00Z",
    },
    auth: { state: "AUTHENTICATED", methods: [], credential_slots: [] },
    configuration: { schema_digest: `sha256:${"b".repeat(64)}`, fields: [], native_config_targets: [] },
    session_options: options,
    input_surface: {
      text: true, images: false, files: false, resources: false,
      slash_commands: commands ? [{ command_id: "review", token: "/review", display_name: "Review" }] : [],
      reference_kinds: [], supports_command_discovery: false, supports_reference_discovery: false,
    },
    native_surface: { native_config_preserved: true, native_extensions_preserved: true, native_subagents_preserved: true },
    capabilities: {} as Descriptor["capabilities"],
    observed_at: "2026-10-18T12:00:00Z",
    expires_at: "2026-10-18T12:05:00Z",
  } as Descriptor;
}

function surfaceFor(d: Descriptor | null) {
  const state: ConversationComposerState = {
    workspaceId: "w", conversationId: "c", agentBindingId: "b", descriptor: d,
    descriptorObservedAt: "2026-10-18T12:00:00Z", enabled: true, leadEligible: true,
    draft: "", attachments: [], nativeTokens: [], selectedOptions: {},
  };
  return deriveConversationAgentSurface(state, NOW);
}

const modelOptions: Descriptor["session_options"] = {
  descriptor_digest: `sha256:${"c".repeat(64)}`,
  options: [
    { option_id: "model", display_name: "Model", semantic_hint: "MODEL", mutable_scope: "NEW_SESSION",
      value_schema: { type: "string", enum: ["p1/m1", "p1/m2"], default: "p1/m2" } },
    { option_id: "effort", display_name: "Effort", semantic_hint: "EFFORT", mutable_scope: "NEW_SESSION",
      value_schema: { type: "string", enum: ["low", "high"] } },
  ],
};

test("model picker lists declared models, marks the default and the selection", () => {
  const d = descriptor(modelOptions);
  const view = deriveModelPicker(surfaceFor(d), d, { model: "p1/m1", effort: "high" });
  assert.equal(view.state, "READY");
  assert.deepEqual(view.items.map(i => [i.providerModelId, i.selected, i.isDefault]), [["p1/m1", true, false], ["p1/m2", false, true]]);
  assert.equal(view.effort?.selected, "high");
  assert.equal(view.announcement, "p1/m1, selected");
});

test("model picker shows no effort row when the descriptor declares none", () => {
  const d = descriptor({ ...modelOptions, options: [modelOptions.options[0]] });
  const view = deriveModelPicker(surfaceFor(d), d, { model: null, effort: null });
  assert.equal(view.effort, null);
  assert.equal(view.message, "Choose a model.");
});

test("model picker renders distinct text for each non-ready state", () => {
  const d = descriptor(modelOptions);
  const messages = new Set<string>();
  for (const load of ["LOADING", "ERROR", "DENIED", "OFFLINE"] as const) {
    const view = deriveModelPicker(surfaceFor(d), d, { model: null, effort: null }, load);
    assert.equal(view.state, load);
    assert.deepEqual(view.items, []);
    messages.add(view.message);
  }
  const stale = deriveModelPicker(surfaceFor(null), null, { model: null, effort: null });
  assert.equal(stale.state, "STALE");
  assert.equal(stale.refreshAvailable, true);
  messages.add(stale.message);
  const empty = descriptor(null);
  const emptyView = deriveModelPicker(surfaceFor(empty), empty, { model: null, effort: null });
  assert.equal(emptyView.state, "EMPTY");
  messages.add(emptyView.message);
  assert.equal(messages.size, 6);
});

test("an unauthenticated or blocked agent exposes no model list", () => {
  const d = descriptor(modelOptions);
  d.auth.state = "NEEDS_AUTH";
  const view = deriveModelPicker(surfaceFor(d), d, { model: null, effort: null });
  assert.equal(view.state, "BLOCKED");
  assert.deepEqual(view.items, []);
});

test("model selection carries the descriptor revision and refuses an undeclared model", () => {
  const d = descriptor(modelOptions);
  const view = deriveModelPicker(surfaceFor(d), d, { model: null, effort: null });
  assert.deepEqual(buildModelSelection(view, d, "p1/m2"), {
    optionId: "model", providerModelId: "p1/m2", descriptorRevision: d.descriptor_digest,
  });
  assert.equal(buildModelSelection(view, d, "other-provider/m9"), null);
});

test("the slash affordance lists only declared commands and says so when none exist", () => {
  const none = deriveCommandList(surfaceFor(descriptor(modelOptions)));
  assert.deepEqual(none.commands, []);
  assert.equal(none.emptyMessage, "No commands declared by this agent");
  const some = deriveCommandList(surfaceFor(descriptor(modelOptions, true)));
  assert.equal(some.commands[0].token, "/review");
  assert.equal(some.emptyMessage, null);
});
