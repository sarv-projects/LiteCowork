/**
 * Composer attachment chips (001-T14, B-08). Only already-ingested Workspace Resources can
 * be attached, as a ResourceRef pinned to the revision the owner saw. The daemon
 * reauthorizes each ref, inlines allowlisted text and refuses everything else before
 * dispatch; this module only keeps the owner's selection honest and bounded.
 */

export const MAX_ATTACHMENTS = 16;

export type AttachmentCandidate = {
  resourceId: string;
  workspaceId: string;
  resourceRevisionId: string;
  displayName: string;
  mediaType: string;
  sizeBytes: number;
};

export type AttachmentChip = AttachmentCandidate;

/** Mirrors the daemon's single-Resource bound (128 KiB) so obviously refused items are not offered. */
export const MAX_ATTACHMENT_BYTES = 128 * 1024;

const TEXT_MEDIA = /^(text\/|application\/(json|xml|x-yaml|yaml|toml|x-ndjson))/i;

/** A conservative picker filter; the daemon stays the authority and may still refuse. */
export function isAttachable(candidate: AttachmentCandidate): boolean {
  return TEXT_MEDIA.test(candidate.mediaType.trim()) && candidate.sizeBytes <= MAX_ATTACHMENT_BYTES;
}

export function addAttachment(chips: readonly AttachmentChip[], candidate: AttachmentCandidate, workspaceId: string): AttachmentChip[] {
  if (candidate.workspaceId !== workspaceId || !isAttachable(candidate)) return [...chips];
  if (chips.some(chip => chip.resourceId === candidate.resourceId)) return [...chips];
  if (chips.length >= MAX_ATTACHMENTS) return [...chips];
  return [...chips, candidate];
}

export function removeAttachment(chips: readonly AttachmentChip[], resourceId: string): AttachmentChip[] {
  return chips.filter(chip => chip.resourceId !== resourceId);
}

/** The wire refs: Workspace, Resource and the exact revision the chip was chosen at. */
export function toResourceRefs(chips: readonly AttachmentChip[], workspaceId: string): { workspace_id: string; resource_id: string; revision_id: string }[] {
  return chips
    .filter(chip => chip.workspaceId === workspaceId)
    .map(chip => ({ workspace_id: chip.workspaceId, resource_id: chip.resourceId, revision_id: chip.resourceRevisionId }));
}

export function attachmentChipLabel(chip: AttachmentChip): string {
  return `Remove attachment ${chip.displayName}`;
}

/** Owner-facing wording for a refused submit; the draft and chips stay in every case. */
export function attachmentRefusalNotice(message: string): string | null {
  if (/changed; reattach/i.test(message)) return "An attached file changed. Remove it and attach the current version.";
  if (/only text/i.test(message)) return "Only text files can be attached to a chat message.";
  if (/too large/i.test(message)) return "An attached file is too large to send.";
  if (/restricted/i.test(message)) return "A restricted file cannot be attached.";
  if (/not found/i.test(message)) return "An attached file is no longer in this Workspace.";
  return null;
}
