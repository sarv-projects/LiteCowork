// Pure update-panel state (019-T12). The panel renders the result of the Feature 011
// quiescence gate and the typed updater outcomes; it never computes quiescence itself.

export type UpdateErrorCode =
  | "ManifestUnreachable"
  | "SignatureInvalid"
  | "Downgrade"
  | "ChannelMismatch"
  | "AlreadyInProgress"
  | "QuiescenceUnavailable"
  | "RuntimeStopFailed"
  | "InstallerFailed";

export type QuiescenceReason = { label: string; status: "running" | "unknown" };

export type UpdateCheckOutcome =
  | { kind: "UpToDate"; current: string }
  | { kind: "Available"; version: string; notesDigest: string; channel: string };

export type UpdateInstallOutcome =
  | { kind: "Refused"; reasons: QuiescenceReason[] }
  | { kind: "InProgress"; requestId: string }
  | { kind: "Handoff"; version: string };

export type UpdateState =
  | { status: "IDLE" }
  | { status: "CHECKING" }
  | { status: "UP_TO_DATE"; current: string }
  | { status: "CHECK_FAILED"; code: UpdateErrorCode }
  | { status: "OFFLINE" }
  | { status: "AVAILABLE"; version: string; notesDigest: string; channel: string }
  | { status: "GATING"; version: string; requestId: string }
  | { status: "REFUSED_NOT_QUIESCENT"; version: string; reasons: QuiescenceReason[]; unavailable: boolean }
  | { status: "INSTALLING"; version: string; requestId: string }
  | { status: "HANDOFF"; version: string }
  | { status: "FAILED"; version: string; code: UpdateErrorCode }
  | { status: "UNSIGNED_DEV" }
  | { status: "LINUX_MANUAL" };

export type UpdateAction =
  | { type: "CHECK_REQUESTED" }
  | { type: "CHECK_RESULT"; outcome: UpdateCheckOutcome }
  | { type: "CHECK_ERROR"; code: UpdateErrorCode; offline: boolean }
  | { type: "INSTALL_REQUESTED"; requestId: string }
  | { type: "INSTALL_RESULT"; outcome: UpdateInstallOutcome }
  | { type: "INSTALL_ERROR"; code: UpdateErrorCode };

export type UpdatePlatform = { os: "windows" | "linux" | "other"; signed: boolean };

export function initialUpdateState(platform: UpdatePlatform): UpdateState {
  if (platform.os === "linux") return { status: "LINUX_MANUAL" };
  if (!platform.signed) return { status: "UNSIGNED_DEV" };
  return { status: "IDLE" };
}

function versionOf(state: UpdateState): string | null {
  return "version" in state ? state.version : null;
}

export function updateReducer(state: UpdateState, action: UpdateAction): UpdateState {
  // Updates are disabled on these rows; no action can leave them.
  if (state.status === "UNSIGNED_DEV" || state.status === "LINUX_MANUAL") return state;
  switch (action.type) {
    case "CHECK_REQUESTED":
      return state.status === "CHECKING" || state.status === "GATING"
        || state.status === "INSTALLING" || state.status === "HANDOFF"
        ? state
        : { status: "CHECKING" };
    case "CHECK_RESULT":
      if (state.status !== "CHECKING") return state;
      return action.outcome.kind === "UpToDate"
        ? { status: "UP_TO_DATE", current: action.outcome.current }
        : {
          status: "AVAILABLE",
          version: action.outcome.version,
          notesDigest: action.outcome.notesDigest,
          channel: action.outcome.channel,
        };
    case "CHECK_ERROR":
      if (state.status !== "CHECKING") return state;
      return action.offline ? { status: "OFFLINE" } : { status: "CHECK_FAILED", code: action.code };
    case "INSTALL_REQUESTED": {
      // Only a visible AVAILABLE offer, a refusal or a failure can start (or retry) an install.
      // A second activation while GATING/INSTALLING is ignored, so one request id is sent.
      if (state.status !== "AVAILABLE" && state.status !== "REFUSED_NOT_QUIESCENT" && state.status !== "FAILED") {
        return state;
      }
      return { status: "GATING", version: versionOf(state) ?? "", requestId: action.requestId };
    }
    case "INSTALL_RESULT": {
      if (state.status !== "GATING") return state;
      const { outcome } = action;
      if (outcome.kind === "Refused") {
        return { status: "REFUSED_NOT_QUIESCENT", version: state.version, reasons: outcome.reasons, unavailable: false };
      }
      if (outcome.kind === "InProgress") {
        return { status: "INSTALLING", version: state.version, requestId: outcome.requestId };
      }
      return { status: "HANDOFF", version: outcome.version };
    }
    case "INSTALL_ERROR": {
      if (state.status !== "GATING" && state.status !== "INSTALLING") return state;
      if (action.code === "QuiescenceUnavailable") {
        return { status: "REFUSED_NOT_QUIESCENT", version: state.version, reasons: [], unavailable: true };
      }
      if (action.code === "AlreadyInProgress") {
        return { status: "INSTALLING", version: state.version, requestId: state.requestId };
      }
      return { status: "FAILED", version: state.version, code: action.code };
    }
  }
}

const ERROR_TEXT: Record<UpdateErrorCode, string> = {
  ManifestUnreachable: "The update server could not be reached.",
  SignatureInvalid: "The update was refused because its signature could not be verified.",
  Downgrade: "The update was refused because it is older than the installed version.",
  ChannelMismatch: "The update was refused because it belongs to a different release channel.",
  AlreadyInProgress: "Another update is already in progress.",
  QuiescenceUnavailable: "LiteCowork could not confirm that nothing is running.",
  RuntimeStopFailed: "The local Runtime did not stop, so the update was not started.",
  InstallerFailed: "The installer did not complete.",
};

export function describeUpdateError(code: UpdateErrorCode): string {
  return ERROR_TEXT[code];
}

/** Specific explanation for every state. Never says "up to date" unless a verified manifest said so. */
export function describeUpdateState(state: UpdateState): string {
  switch (state.status) {
    case "IDLE": return "Updates have not been checked in this session.";
    case "CHECKING": return "Checking for updates.";
    case "UP_TO_DATE": return `LiteCowork ${state.current} is up to date.`;
    case "CHECK_FAILED": return `Could not check for updates. ${describeUpdateError(state.code)}`;
    case "OFFLINE": return "Could not check for updates because this device is offline.";
    case "AVAILABLE": return `Version ${state.version} is available on the ${state.channel} channel.`;
    case "GATING": return "Checking that nothing is running.";
    case "REFUSED_NOT_QUIESCENT": return state.unavailable
      ? `Update not started. ${describeUpdateError("QuiescenceUnavailable")}`
      : "Update not started because work is still running or could not be confirmed.";
    case "INSTALLING": return `Installing version ${state.version}. Do not close LiteCowork.`;
    case "HANDOFF": return `Version ${state.version} is ready. LiteCowork will restart.`;
    case "FAILED": return `Update to ${state.version} failed. ${describeUpdateError(state.code)}`;
    case "UNSIGNED_DEV": return "Updates are disabled because this is an unsigned development build.";
    case "LINUX_MANUAL": return "Update by installing the new package.";
  }
}

/** Install is enabled only for an offered update; otherwise a visible reason is returned. */
export function installAvailability(state: UpdateState): { enabled: true } | { enabled: false; reason: string } {
  if (state.status === "AVAILABLE") return { enabled: true };
  if (state.status === "REFUSED_NOT_QUIESCENT" || state.status === "FAILED") return { enabled: true };
  return { enabled: false, reason: `Install is unavailable: ${describeUpdateState(state)}` };
}

export function canCheck(state: UpdateState): boolean {
  return state.status !== "CHECKING" && state.status !== "GATING" && state.status !== "INSTALLING"
    && state.status !== "HANDOFF" && state.status !== "UNSIGNED_DEV" && state.status !== "LINUX_MANUAL";
}

export type UpdateApi = {
  checkForUpdate(): Promise<UpdateCheckOutcome>;
  installUpdate(input: { expectedVersion: string; requestId: string }): Promise<UpdateInstallOutcome>;
};

export class UpdateCommandError extends Error {
  readonly code: UpdateErrorCode;
  constructor(code: UpdateErrorCode) {
    super(code);
    this.name = "UpdateCommandError";
    this.code = code;
  }
}

/**
 * Drives the commands. `getState` must return the latest reducer state. The install command is
 * sent only when the reducer actually enters GATING, so a double activation sends one request id.
 */
export function createUpdateController(options: {
  api: UpdateApi;
  dispatch: (action: UpdateAction) => void;
  getState: () => UpdateState;
  newRequestId: () => string;
  isOffline: () => boolean;
}) {
  let installInFlight = false;
  let checkInFlight = false;
  return {
    async check(): Promise<void> {
      if (checkInFlight || !canCheck(options.getState())) return;
      checkInFlight = true;
      options.dispatch({ type: "CHECK_REQUESTED" });
      try {
        options.dispatch({ type: "CHECK_RESULT", outcome: await options.api.checkForUpdate() });
      } catch (error) {
        const code = error instanceof UpdateCommandError ? error.code : "ManifestUnreachable";
        options.dispatch({ type: "CHECK_ERROR", code, offline: options.isOffline() });
      } finally {
        checkInFlight = false;
      }
    },
    async install(): Promise<void> {
      const state = options.getState();
      if (installInFlight || !installAvailability(state).enabled) return;
      const version = versionOf(state);
      if (!version) return;
      installInFlight = true;
      const requestId = options.newRequestId();
      options.dispatch({ type: "INSTALL_REQUESTED", requestId });
      try {
        const outcome = await options.api.installUpdate({ expectedVersion: version, requestId });
        options.dispatch({ type: "INSTALL_RESULT", outcome });
      } catch (error) {
        const code = error instanceof UpdateCommandError ? error.code : "InstallerFailed";
        options.dispatch({ type: "INSTALL_ERROR", code });
      } finally {
        installInFlight = false;
      }
    },
  };
}
