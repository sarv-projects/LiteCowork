import assert from "node:assert/strict";
import test from "node:test";
import {
  isTaskPlanningReadinessFor,
  planningBlockerLabel,
  planningReadinessNoBlockersMessage,
  type TaskPlanningReadinessExpectation,
  type TaskPlanningReadinessView,
} from "../src/tasks/planning-readiness.ts";

const expected: TaskPlanningReadinessExpectation = {
  taskId: "task-1",
  taskVersion: 7,
  taskSpecRevision: 3,
  taskStatus: "READY",
};

const readiness: TaskPlanningReadinessView = {
  ...expected,
  observedAt: "2026-10-09T12:00:00Z",
  dispatchAvailable: false,
  planningStarted: false,
  agentSessionStarted: false,
  planCreated: false,
  blockers: ["TASK_ISOLATION_UNAVAILABLE"],
};

test("accepts a read-only readiness projection for the currently visible Task", () => {
  assert.equal(isTaskPlanningReadinessFor(readiness, expected), true);
});

test("accepts and explains the Windows Environment provider blocker", () => {
  const windowsReadiness = {
    ...readiness,
    blockers: ["WINDOWS_ENVIRONMENT_PROVIDER_UNAVAILABLE"],
  };
  assert.equal(isTaskPlanningReadinessFor(windowsReadiness, expected), true);
  assert.equal(
    planningBlockerLabel("WINDOWS_ENVIRONMENT_PROVIDER_UNAVAILABLE"),
    "Task execution is not available on Windows yet: no qualified Windows Environment provider.",
  );
});

for (const [field, value] of [
  ["taskId", "task-2"],
  ["taskVersion", 8],
  ["taskSpecRevision", 4],
  ["taskStatus", "RUNNING"],
] as const) {
  test(`rejects readiness with mismatched ${field}`, () => {
    assert.equal(isTaskPlanningReadinessFor({ ...readiness, [field]: value }, expected), false);
  });
}

for (const field of ["dispatchAvailable", "planningStarted", "agentSessionStarted", "planCreated"] as const) {
  test(`rejects readiness when ${field} claims execution occurred`, () => {
    assert.equal(isTaskPlanningReadinessFor({ ...readiness, [field]: true }, expected), false);
  });
}

test("rejects malformed blocker data", () => {
  assert.equal(isTaskPlanningReadinessFor({ ...readiness, blockers: ["UNKNOWN_BLOCKER"] }, expected), false);
});

test("zero blockers still explicitly says planning dispatch is unavailable", () => {
  const message = planningReadinessNoBlockersMessage([]);
  assert.ok(message);
  assert.match(message, /Planning dispatch is unavailable/);
  assert.match(message, /does not authorize or start work/);
});

test("blockers are rendered by their separate list and need no empty-list message", () => {
  assert.equal(planningReadinessNoBlockersMessage(["TASK_ISOLATION_UNAVAILABLE"]), null);
});
