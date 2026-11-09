export const MAX_WORKSPACE_NAME_CHARS = 200;

export type WorkspaceRenameFailure = "STALE" | "ARCHIVED" | "INVALID" | "OTHER";

/** Mirrors the Operator rules: trimmed, 1 to 200 characters, no control characters. */
export function validateWorkspaceName(name: string): { ok: true; name: string } | { ok: false; reason: string } {
  const trimmed = name.trim();
  if (trimmed.length === 0) return { ok: false, reason: "Enter a Workspace name." };
  if ([...trimmed].length > MAX_WORKSPACE_NAME_CHARS) {
    return { ok: false, reason: `Workspace names are limited to ${MAX_WORKSPACE_NAME_CHARS} characters.` };
  }
  // eslint-disable-next-line no-control-regex
  if (/[\u0000-\u001f\u007f-\u009f]/.test(trimmed)) {
    return { ok: false, reason: "Workspace names cannot contain control characters." };
  }
  return { ok: true, name: trimmed };
}

/** The desktop bridge surfaces Operator refusals as `message [CODE; ...]`. */
export function classifyRenameFailure(message: string): WorkspaceRenameFailure {
  if (message.includes("[STALE_WORKSPACE_VERSION")) return "STALE";
  if (message.includes("[WORKSPACE_ARCHIVED")) return "ARCHIVED";
  if (message.includes("[WORKSPACE_NAME_INVALID") || message.includes("Workspace rename request is invalid")) {
    return "INVALID";
  }
  return "OTHER";
}

export type PendingRename = { workspaceId: string; version: number; name: string; requestId: string };

/**
 * A retry of the same Workspace, version and name reuses the same Idempotency-Key so an
 * ambiguous first attempt replays its receipt; any change gets a new key.
 */
export function nextRenameRequest(
  pending: PendingRename | null,
  workspaceId: string,
  version: number,
  name: string,
  newId: () => string,
): PendingRename {
  if (pending && pending.workspaceId === workspaceId && pending.version === version && pending.name === name) {
    return pending;
  }
  return { workspaceId, version, name, requestId: newId() };
}

export function renameFailureMessage(failure: WorkspaceRenameFailure, latestName: string | null): string {
  switch (failure) {
    case "STALE":
      return latestName
        ? `This Workspace changed elsewhere. The latest name is "${latestName}". Your draft is kept; review it and rename again.`
        : "This Workspace changed elsewhere. Refresh and rename again; your draft is kept.";
    case "ARCHIVED":
      return "Archived Workspaces are read-only and cannot be renamed.";
    case "INVALID":
      return "The Workspace name was refused. Use 1 to 200 characters without control characters.";
    default:
      return "Workspace could not be renamed. Nothing was changed; try again.";
  }
}
