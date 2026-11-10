import assert from "node:assert/strict";
import test from "node:test";
import {
  classifyRenameFailure,
  nextRenameRequest,
  renameFailureMessage,
  validateWorkspaceName,
} from "../src/settings/workspace-rename.ts";

test("names are trimmed and bounded like the Operator", () => {
  assert.deepEqual(validateWorkspaceName("  Project  "), { ok: true, name: "Project" });
  assert.equal(validateWorkspaceName("   ").ok, false);
  assert.equal(validateWorkspaceName("a".repeat(200)).ok, true);
  assert.equal(validateWorkspaceName("a".repeat(201)).ok, false);
  assert.equal(validateWorkspaceName("bad\u0007name").ok, false);
});

test("Operator refusals map to distinct UI failures", () => {
  assert.equal(
    classifyRenameFailure("Workspace changed; refresh before renaming [STALE_WORKSPACE_VERSION; retryable=false; reference=cor_1]"),
    "STALE",
  );
  assert.equal(classifyRenameFailure("Archived Workspaces are read-only [WORKSPACE_ARCHIVED; retryable=false; reference=cor_1]"), "ARCHIVED");
  assert.equal(classifyRenameFailure("x [WORKSPACE_NAME_INVALID; retryable=false; reference=c]"), "INVALID");
  assert.equal(classifyRenameFailure("Workspace rename request did not complete"), "OTHER");
});

test("a stale conflict names the latest name and keeps the draft", () => {
  const message = renameFailureMessage("STALE", "Latest name");
  assert.match(message, /Latest name/);
  assert.match(message, /draft is kept/);
});

test("archived refusal is read-only wording", () => {
  assert.match(renameFailureMessage("ARCHIVED", null), /read-only/);
});

test("same workspace, version and name reuse the idempotency key; any change gets a new one", () => {
  let counter = 0;
  const newId = () => `id-${++counter}`;
  const first = nextRenameRequest(null, "ws", 1, "A", newId);
  assert.equal(nextRenameRequest(first, "ws", 1, "A", newId).requestId, first.requestId);
  assert.notEqual(nextRenameRequest(first, "ws", 1, "B", newId).requestId, first.requestId);
  assert.notEqual(nextRenameRequest(first, "ws", 2, "A", newId).requestId, first.requestId);
  assert.notEqual(nextRenameRequest(first, "other", 1, "A", newId).requestId, first.requestId);
});
