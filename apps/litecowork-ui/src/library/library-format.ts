import type { WorkspaceRootView } from "../App";

export function formatBytes(size: number): string {
  if (size === 0) return "0 B";
  if (size < 1024) return `${size} B`;
  return size < 1024 * 1024 ? `${Math.round(size / 1024)} KB` : `${(size / (1024 * 1024)).toFixed(1)} MB`;
}

export function supportsTextPreview(mediaType: string, sizeBytes: number): boolean {
  const normalized = mediaType.toLowerCase().split(";", 1)[0].trim();
  const textLike = normalized.startsWith("text/") || [
    "application/json",
    "application/xml",
    "application/yaml",
    "application/x-yaml",
    "application/javascript",
  ].includes(normalized);
  return textLike && sizeBytes <= 1024 * 1024;
}

export function workspaceRootStatusLabel(status: WorkspaceRootView["status"]): string {
  switch (status) {
    case "ACTIVE": return "Active · folder watching and indexing are not active in this build.";
    case "PAUSED": return "Paused · watching is off in this build. Identity is checked at startup; restart LiteCowork before resuming after moving or replacing this folder.";
    case "UNAVAILABLE": return "Unavailable · folder identity could not be verified at startup. Check the original folder and restart LiteCowork to retry.";
    case "REVOKED": return "Access removed.";
  }
}
