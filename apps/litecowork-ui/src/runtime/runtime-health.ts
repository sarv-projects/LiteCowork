// Derives the Settings > Runtime health rows from observed Runtime state only.
// Nothing here is inferred from a click: every value comes from the readiness
// probe (`operatorReady`) or the lifecycle view (`recoveryState`).

export type RuntimeHealthInput = {
  operatorReady: boolean;
  recoveryState: string | null;
};

export type RuntimeHealthRows = {
  transport: string;
  operator: string;
  recoveryState: string;
  execution: string;
  executionReady: boolean;
};

export function localIpcTransportLabel(platform: string): string {
  return /^win/i.test(platform) ? "Local IPC (named pipe)" : "Local IPC (socket)";
}

export function deriveRuntimeHealthRows(
  input: RuntimeHealthInput,
  platform: string,
): RuntimeHealthRows {
  const transport = localIpcTransportLabel(platform);
  const recoveryState = input.recoveryState ?? "UNKNOWN";
  const executionReady = input.operatorReady && recoveryState === "READY";
  let execution = "Closed";
  if (executionReady) {
    execution = "Open";
  } else if (input.operatorReady && recoveryState === "DEGRADED") {
    execution = "Closed (degraded; Operator commands remain available)";
  } else if (!input.operatorReady) {
    execution = "Closed (Operator not serving)";
  }
  return {
    transport,
    operator: input.operatorReady ? `Serving over ${transport}` : "Not serving",
    recoveryState,
    execution,
    executionReady,
  };
}
