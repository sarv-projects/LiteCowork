export type PlanningBlocker =
  | "TASK_STATE_NOT_ELIGIBLE"
  | "PLAN_ALREADY_ACCEPTED"
  | "CURRENT_LEAD_OR_ENDPOINT_UNAVAILABLE"
  | "TASK_ISOLATION_UNAVAILABLE"
  | "NATIVE_CAPABILITIES_UNMEDIATED"
  | "PROTOCOL_UNQUALIFIED"
  | "PROCESS_CONTAINMENT_UNQUALIFIED"
  | "PLANNING_CONTEXT_RESOURCE_UNAVAILABLE"
  | "SESSION_SETTLEMENT_UNAVAILABLE"
  | "PROVIDER_UNSUPPORTED"
  | "WINDOWS_ENVIRONMENT_PROVIDER_UNAVAILABLE";

export type TaskStatus =
  | "READY"
  | "RUNNING"
  | "WAITING_USER"
  | "BLOCKED"
  | "VERIFYING"
  | "NEEDS_USER"
  | "INCOMPLETE"
  | "PAUSE_REQUESTED"
  | "PAUSED"
  | "COMPLETED"
  | "FAILED"
  | "CANCEL_REQUESTED"
  | "CANCELLED";

export type TaskPlanningReadinessView = {
  taskId: string;
  taskVersion: number;
  taskSpecRevision: number;
  taskStatus: TaskStatus;
  observedAt: string;
  dispatchAvailable: false;
  planningStarted: false;
  agentSessionStarted: false;
  planCreated: false;
  blockers: PlanningBlocker[];
};

export type TaskPlanningReadinessExpectation = Pick<
  TaskPlanningReadinessView,
  "taskId" | "taskVersion" | "taskSpecRevision" | "taskStatus"
>;

const PLANNING_BLOCKERS: readonly PlanningBlocker[] = [
  "TASK_STATE_NOT_ELIGIBLE",
  "PLAN_ALREADY_ACCEPTED",
  "CURRENT_LEAD_OR_ENDPOINT_UNAVAILABLE",
  "TASK_ISOLATION_UNAVAILABLE",
  "NATIVE_CAPABILITIES_UNMEDIATED",
  "PROTOCOL_UNQUALIFIED",
  "PROCESS_CONTAINMENT_UNQUALIFIED",
  "PLANNING_CONTEXT_RESOURCE_UNAVAILABLE",
  "SESSION_SETTLEMENT_UNAVAILABLE",
  "PROVIDER_UNSUPPORTED",
  "WINDOWS_ENVIRONMENT_PROVIDER_UNAVAILABLE",
];

const PLANNING_BLOCKER_LABELS: Record<PlanningBlocker, string> = {
  TASK_STATE_NOT_ELIGIBLE: "The Task state is not eligible for planning.",
  PLAN_ALREADY_ACCEPTED: "A plan is already saved for this Task.",
  CURRENT_LEAD_OR_ENDPOINT_UNAVAILABLE: "The selected lead or a fresh compatible local endpoint is unavailable.",
  TASK_ISOLATION_UNAVAILABLE: "Task-specific execution isolation is not available.",
  NATIVE_CAPABILITIES_UNMEDIATED: "The agent's native capabilities are not yet mediated by LiteCowork.",
  PROTOCOL_UNQUALIFIED: "The planning protocol has not been qualified for this setup.",
  PROCESS_CONTAINMENT_UNQUALIFIED: "Process containment has not been qualified for this setup.",
  PLANNING_CONTEXT_RESOURCE_UNAVAILABLE: "Required planning context is unavailable.",
  SESSION_SETTLEMENT_UNAVAILABLE: "Safe planning-session settlement is not available.",
  PROVIDER_UNSUPPORTED: "This provider is not supported for local planning.",
  WINDOWS_ENVIRONMENT_PROVIDER_UNAVAILABLE: "Task execution is not available on Windows yet: no qualified Windows Environment provider.",
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isPlanningBlocker(value: unknown): value is PlanningBlocker {
  return typeof value === "string" && PLANNING_BLOCKERS.includes(value as PlanningBlocker);
}

export function planningBlockerLabel(blocker: PlanningBlocker): string {
  return PLANNING_BLOCKER_LABELS[blocker];
}

/**
 * Confirms the readiness response still describes the visible Task and remains
 * a diagnostic only. A malformed or execution-capable response is rejected.
 */
export function isTaskPlanningReadinessFor(
  value: unknown,
  expected: TaskPlanningReadinessExpectation,
): value is TaskPlanningReadinessView {
  if (!isRecord(value)) return false;
  return value.taskId === expected.taskId
    && value.taskVersion === expected.taskVersion
    && value.taskSpecRevision === expected.taskSpecRevision
    && value.taskStatus === expected.taskStatus
    && typeof value.observedAt === "string"
    && value.dispatchAvailable === false
    && value.planningStarted === false
    && value.agentSessionStarted === false
    && value.planCreated === false
    && Array.isArray(value.blockers)
    && value.blockers.every(isPlanningBlocker);
}

/** Returns the diagnostic's empty-blocker message without implying readiness. */
export function planningReadinessNoBlockersMessage(
  blockers: readonly PlanningBlocker[],
): string | null {
  return blockers.length === 0
    ? "No local preflight blocker was observed. Planning dispatch is unavailable; this result does not authorize or start work."
    : null;
}
