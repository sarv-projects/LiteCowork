import { invoke } from "@tauri-apps/api/core";
import {
  UpdateCommandError,
  type QuiescenceReason,
  type UpdateApi,
  type UpdateCheckOutcome,
  type UpdateErrorCode,
  type UpdateInstallOutcome,
} from "./update-state.ts";

const ERROR_CODES: readonly UpdateErrorCode[] = [
  "ManifestUnreachable", "SignatureInvalid", "Downgrade", "ChannelMismatch",
  "AlreadyInProgress", "QuiescenceUnavailable", "RuntimeStopFailed", "InstallerFailed",
];

function record(value: unknown): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new UpdateCommandError("ManifestUnreachable");
  }
  return value as Record<string, unknown>;
}

function text(value: unknown, code: UpdateErrorCode): string {
  if (typeof value !== "string" || value.length === 0 || value.length > 256) throw new UpdateCommandError(code);
  return value;
}

/** Tauri rejects with the typed UpdateError name; anything else is not trusted as a typed cause. */
export function toUpdateCommandError(error: unknown, fallback: UpdateErrorCode): UpdateCommandError {
  if (error instanceof UpdateCommandError) return error;
  const name = typeof error === "string" ? error
    : typeof error === "object" && error !== null && "code" in error ? (error as { code: unknown }).code : null;
  const known = ERROR_CODES.find((code) => code === name);
  return new UpdateCommandError(known ?? fallback);
}

export function parseCheckOutcome(raw: unknown): UpdateCheckOutcome {
  const value = record(raw);
  if (value.kind === "UpToDate") return { kind: "UpToDate", current: text(value.current, "ManifestUnreachable") };
  if (value.kind === "Available") {
    return {
      kind: "Available",
      version: text(value.version, "ManifestUnreachable"),
      notesDigest: text(value.notes_digest, "ManifestUnreachable"),
      channel: text(value.channel, "ManifestUnreachable"),
    };
  }
  throw new UpdateCommandError("ManifestUnreachable");
}

function parseReasons(raw: unknown): QuiescenceReason[] {
  if (!Array.isArray(raw)) throw new UpdateCommandError("QuiescenceUnavailable");
  return raw.slice(0, 50).map((item) => {
    const value = record(item);
    const status = value.status === "running" ? "running" : "unknown";
    return { label: text(value.label, "QuiescenceUnavailable"), status };
  });
}

export function parseInstallOutcome(raw: unknown): UpdateInstallOutcome {
  const value = record(raw);
  if (value.kind === "Refused") return { kind: "Refused", reasons: parseReasons(value.reasons) };
  if (value.kind === "InProgress") return { kind: "InProgress", requestId: text(value.request_id, "InstallerFailed") };
  if (value.kind === "Handoff") return { kind: "Handoff", version: text(value.version, "InstallerFailed") };
  throw new UpdateCommandError("InstallerFailed");
}

export function desktopUpdateApi(): UpdateApi {
  return {
    async checkForUpdate() {
      try {
        return parseCheckOutcome(await invoke<unknown>("check_for_update"));
      } catch (error) {
        throw toUpdateCommandError(error, "ManifestUnreachable");
      }
    },
    async installUpdate({ expectedVersion, requestId }) {
      try {
        return parseInstallOutcome(await invoke<unknown>("install_update", { expectedVersion, requestId }));
      } catch (error) {
        throw toUpdateCommandError(error, "InstallerFailed");
      }
    },
  };
}
