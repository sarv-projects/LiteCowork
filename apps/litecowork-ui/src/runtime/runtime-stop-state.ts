// Pure presentation state for the stop preview and update quiescence gate (011-T14).
// The UI never computes quiescence: it renders the verdict of the authenticated
// previewLocalRuntimeStop read and refuses anything but a positive QUIESCENT verdict.

export type StopVerdict = "QUIESCENT" | "BLOCKED" | "UNKNOWN";

export type RuntimeStopPreviewView = {
  runtimeId: string;
  runtimeIncarnationId: string;
  observedAt: string;
  admittedTurnIds: string[];
  admittedAttemptIds: string[];
  unresolvedEffectIds: string[];
  executionLeaseIds: string[];
  environmentControlLeaseIds: string[];
  providerReferenceIds: string[];
  pendingCleanupIds: string[];
  verdict: StopVerdict;
  blockers: { blockerId: string; code: string; safeMessage: string; resolutionHint: string }[];
};

export type StopPreviewState =
  | { status: "IDLE" }
  | { status: "LOADING" }
  | { status: "ERROR"; message: string }
  | { status: "READY"; preview: RuntimeStopPreviewView };

/** requestLocalRuntimeStop is not mounted (011-T13 blocked), so Stop safely can never be enabled. */
export const STOP_COMMAND_AVAILABLE = false;

export type StopSummaryItem = { label: string; count: number };

export type StopSummary = {
  verdictLabel: string;
  items: StopSummaryItem[];
  reasons: string[];
  stopEnabled: boolean;
  stopReason: string;
  updateGate: { allowed: boolean; message: string };
};

function items(preview: RuntimeStopPreviewView): StopSummaryItem[] {
  return [
    { label: "Active Conversation turns", count: preview.admittedTurnIds.length },
    { label: "Active Task Attempts", count: preview.admittedAttemptIds.length },
    { label: "Unreconciled external Effects", count: preview.unresolvedEffectIds.length },
    { label: "Held execution leases", count: preview.executionLeaseIds.length },
    { label: "Held environment control leases", count: preview.environmentControlLeaseIds.length },
    { label: "Provider references", count: preview.providerReferenceIds.length },
    { label: "Pending cleanup", count: preview.pendingCleanupIds.length },
  ].filter((item) => item.count > 0);
}

export function summarizeStopPreview(
  state: StopPreviewState,
  expected: { runtimeId: string | null; incarnationId: string | null },
): StopSummary {
  const refused = (message: string): StopSummary => ({
    verdictLabel: "Not checked",
    items: [],
    reasons: [],
    stopEnabled: false,
    stopReason: message,
    updateGate: { allowed: false, message },
  });
  if (state.status === "IDLE") return refused("Preview the stop first. Nothing has been checked.");
  if (state.status === "LOADING") return refused("Checking what is running.");
  if (state.status === "ERROR") return refused(`The stop preview is unavailable: ${state.message}`);
  const { preview } = state;
  if (
    !expected.runtimeId || !expected.incarnationId
    || preview.runtimeId !== expected.runtimeId
    || preview.runtimeIncarnationId !== expected.incarnationId
  ) {
    return refused("The preview belongs to a different Runtime incarnation. Preview again.");
  }
  const reasons = preview.blockers.map((blocker) => blocker.safeMessage);
  const listed = items(preview);
  if (preview.verdict === "QUIESCENT") {
    return {
      verdictLabel: "Nothing is running",
      items: listed,
      reasons,
      stopEnabled: STOP_COMMAND_AVAILABLE,
      stopReason: STOP_COMMAND_AVAILABLE ? "" : "Stop safely is unavailable until the authenticated stop command exists.",
      updateGate: { allowed: true, message: "Nothing is running, so an update may proceed." },
    };
  }
  const verdictLabel = preview.verdict === "BLOCKED" ? "Work is still running" : "Cannot confirm that nothing is running";
  const gateMessage = preview.verdict === "BLOCKED"
    ? "Update refused: work is still running."
    : "Update refused: LiteCowork cannot confirm that nothing is running.";
  return {
    verdictLabel,
    items: listed,
    reasons,
    stopEnabled: false,
    stopReason: `Stop safely is disabled: ${verdictLabel.toLowerCase()}.`,
    updateGate: { allowed: false, message: gateMessage },
  };
}

/** Window close is never a Runtime stop; the label must come from the observed lifecycle only. */
export function runtimeStoppedLabel(recoveryState: string | null | undefined): string | null {
  return recoveryState === "STOPPED" ? "Runtime stopped" : null;
}
