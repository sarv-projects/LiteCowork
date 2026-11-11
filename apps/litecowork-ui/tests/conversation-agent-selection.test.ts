import assert from "node:assert/strict";
import test from "node:test";
import { buildConversationAgentChoices } from "../src/conversations/conversation-agent-selection.ts";

const bindings = [
  { agentBindingId: "binding-eligible", workspaceId: "workspace-a", agentProfileId: "profile-a", enabled: true, leadEligible: true },
  { agentBindingId: "binding-disabled", workspaceId: "workspace-a", agentProfileId: "profile-b", enabled: false, leadEligible: true },
  { agentBindingId: "binding-delegate-only", workspaceId: "workspace-a", agentProfileId: "profile-c", enabled: true, leadEligible: false },
  { agentBindingId: "binding-other-workspace", workspaceId: "workspace-b", agentProfileId: "profile-d", enabled: true, leadEligible: true },
];

const profiles = [
  { agentProfileId: "profile-a", displayName: "OpenCode", providerKey: "OPENCODE" },
  { agentProfileId: "profile-b", displayName: "Disabled Agent", providerKey: "OPENCODE" },
  { agentProfileId: "profile-c", displayName: "Subagent Only", providerKey: "OPENCODE" },
  { agentProfileId: "profile-d", displayName: "Other Workspace Agent", providerKey: "OPENCODE" },
];

test("Conversation choices include only enabled lead bindings from this Workspace", () => {
  assert.deepEqual(
    buildConversationAgentChoices("workspace-a", bindings, profiles, null),
    [{ agentBindingId: "binding-eligible", label: "OpenCode", selectable: true }],
  );
});

test("a stale current override remains visible only so the user can clear it", () => {
  assert.deepEqual(
    buildConversationAgentChoices("workspace-a", bindings, profiles, "binding-disabled"),
    [
      { agentBindingId: "binding-eligible", label: "OpenCode", selectable: true },
      { agentBindingId: "binding-disabled", label: "Unavailable selected agent", selectable: false },
    ],
  );
});

test("duplicate profile names are disambiguated by binding identity", () => {
  assert.deepEqual(
    buildConversationAgentChoices(
      "workspace-a",
      [
        bindings[0]!,
        { ...bindings[0]!, agentBindingId: "binding-eligible-2" },
      ],
      profiles,
      null,
    ),
    [
      { agentBindingId: "binding-eligible", label: "OpenCode \u00b7 binding-eligible", selectable: true },
      { agentBindingId: "binding-eligible-2", label: "OpenCode \u00b7 binding-eligible-2", selectable: true },
    ],
  );
});

test("profiles without a Conversation dispatch arm are not selectable", () => {
  assert.deepEqual(
    buildConversationAgentChoices(
      "workspace-a",
      [bindings[0]!],
      [{ agentProfileId: "profile-a", displayName: "Codex", providerKey: "CODEX" }],
      null,
    ),
    [],
  );
});
