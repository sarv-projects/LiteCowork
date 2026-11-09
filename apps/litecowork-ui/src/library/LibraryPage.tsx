import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ArtifactLibrary } from "../artifacts/ArtifactLibrary";
import { desktopArtifactApi } from "../artifacts/desktop-artifact-api";
import type { PinnedResourceRef } from "../artifacts/artifact-api";
import { WorkspaceRootObservations, type WorkspaceRootObservationPage, type WorkspaceRootObservationRequest } from "../workspaces/WorkspaceRootObservations";
import { ZipIntakeNotice } from "../resources/ZipIntakeNotice";
import { resourceIndexAction} from "../resources/resource-intake";
import { isCurrentWorkspaceSearchRequest } from "../resources/resource-search-request";
import { ResourceRevisionEditor } from "../resources/ResourceRevisionEditor";
import { ResourceSourceReferences } from "../resources/ResourceSourceReferences";
import { resourceSearchSourceReferences } from "../resources/resource-source-references";
import { performResourceIndexRebuild } from "../resources/resource-index-rebuild.ts";
import { type SourceHighlightResult } from "../resources/source-span-highlights";
import {
  isCurrentWorkspaceRootRequest,
  performWorkspaceRootStatusAction,
  resourceLocationAvailabilityLabel,
} from "../workspaces/workspace-root-status.ts";
import {
  acceptWorkspaceRootScanResponse,
  isCurrentWorkspaceRootScan,
  isWorkspaceRootScannable,
  workspaceRootScanFailure,
  workspaceRootScanAnnouncement,
  workspaceRootScanRequest,
  type WorkspaceRootScanRequest,
} from "../workspaces/workspace-root-scan.ts";
import {
  clearWorkspaceRootMutationRequest,
  isWorkspaceRootAddResponse,
  isWorkspaceRootRevokeResponse,
  workspaceRootMutationRequest,
} from "../workspaces/workspace-root-mutations.ts";
import type { ContextDocumentCreateMetadata, PreviewSearchPin, ResourceContentScanView, ResourceSearchPageView, ResourceSearchResultView, ResourceTextIndexRebuildView, ResourceUploadResumeForget, ResourceView, TaskInputSelection, WorkspaceRootPageView, WorkspaceRootScanFeedback, WorkspaceRootView } from "../App";
import { formatBytes, supportsTextPreview, workspaceRootStatusLabel } from "./library-format";

export function LibraryPage({ selectedWorkspaceId, onSelectWorkspace, selectedWorkspaceIdRef, artifactApi, onOpenArtifactSource, rootStatusRequestKeys, resourceIndexRequestKeys, workspaceVersion, operatorReady, resources, onRevisionCommitted, onDraftSaveFailure, onRefreshResources, selectedTaskInputs, onToggleTaskInput, onClearTaskInputs, onUseSelectedInTask, nextCursor, pageBusy, onLoadMore, busy, pausePending, onPauseImport, message, listFailed, uploadResumeForget, onForgetUploadResume, onFiles, onPreview, onClosePreview, previewResourceId, previewResourceName, preview, previewHighlights, previewError }: { selectedWorkspaceId: string; onSelectWorkspace: (workspaceId: string) => void; selectedWorkspaceIdRef: { current: string }; artifactApi: ReturnType<typeof desktopArtifactApi>; onOpenArtifactSource: (ref: PinnedResourceRef) => void; rootStatusRequestKeys: { current: Map<string, string> }; resourceIndexRequestKeys: { current: Map<string, string> }; workspaceVersion: number | null; operatorReady: boolean; resources: ResourceView[]; onRevisionCommitted: (resource: ResourceView) => void; onDraftSaveFailure: (message: string) => void; onRefreshResources: () => void; selectedTaskInputs: TaskInputSelection[]; onToggleTaskInput: (resource: { resourceId: string; resourceRevisionId: string; displayName: string }) => void; onClearTaskInputs: () => void; onUseSelectedInTask: () => void; nextCursor: string | null; pageBusy: boolean; onLoadMore: () => Promise<void>; busy: boolean; pausePending: boolean; onPauseImport: () => void; message: string | null; listFailed: boolean; uploadResumeForget: ResourceUploadResumeForget | null; onForgetUploadResume: () => void; onFiles: (files: FileList | File[] | null, contextDocument?: ContextDocumentCreateMetadata | null) => Promise<void>; onPreview: (resourceId: string, displayName: string, revisionId: string, searchPin?: PreviewSearchPin) => Promise<void>; onClosePreview: () => void; previewResourceId: string | null; previewResourceName: string | null; preview: string | null; previewHighlights: SourceHighlightResult | null; previewError: string | null }) {
  const folderInput = useRef<HTMLInputElement>(null);
  const rootListGeneration = useRef(0);
  const [workspaceRoots, setWorkspaceRoots] = useState<WorkspaceRootView[]>([]);
  const [rootNextCursor, setRootNextCursor] = useState<string | null>(null);
  const [rootBusy, setRootBusy] = useState(false);
  const [rootAdding, setRootAdding] = useState(false);
  const [rootRevokingId, setRootRevokingId] = useState<string | null>(null);
  const [rootStatusAction, setRootStatusAction] = useState<{ workspaceRootId: string; action: "pause" | "resume" } | null>(null);
  const rootScanRequests = useRef(new Map<string, string>());
  const rootScanActionRef = useRef<{ request: WorkspaceRootScanRequest; feedbackKey: string } | null>(null);
  const [rootScanAction, setRootScanAction] = useState<{ request: WorkspaceRootScanRequest; feedbackKey: string } | null>(null);
  const [rootScanFeedback, setRootScanFeedback] = useState<Record<string, WorkspaceRootScanFeedback>>({});
  const [rootError, setRootError] = useState<string | null>(null);
  const [rootMessage, setRootMessage] = useState<string | null>(null);
  const workspaceRootsRef = useRef(workspaceRoots);
  workspaceRootsRef.current = workspaceRoots;
  const searchRequest = useRef(0);
  const [resourceIndexRefreshNonce, setResourceIndexRefreshNonce] = useState(0);
  const [query, setQuery] = useState("");
  const [searchResults, setSearchResults] = useState<ResourceSearchResultView[]>([]);
  const [searchCursor, setSearchCursor] = useState<string | null>(null);
  const [searchBusy, setSearchBusy] = useState(false);
  const [searchError, setSearchError] = useState<string | null>(null);
  const [searchMode, setSearchMode] = useState<"METADATA" | "ON_DEMAND_CONTENT" | "INDEXED_CONTENT">("METADATA");
  const [searchKind, setSearchKind] = useState("");
  const [searchFreshness, setSearchFreshness] = useState("");
  const [contentScanInfo, setContentScanInfo] = useState<ResourceContentScanView | null>(null);
  const [rebuildBusyResourceId, setRebuildBusyResourceId] = useState<string | null>(null);
  const [saveAsBusyResourceId, setSaveAsBusyResourceId] = useState<string | null>(null);
  const [saveAsMessage, setSaveAsMessage] = useState<string | null>(null);
  const [saveAsError, setSaveAsError] = useState<string | null>(null);
  const [indexActionMessage, setIndexActionMessage] = useState<string | null>(null);
  const [indexActionError, setIndexActionError] = useState<string | null>(null);
  const [indexRetryPin, setIndexRetryPin] = useState<{ workspaceId: string; resourceId: string; revisionId: string; contentDigest: string } | null>(null);
  const [dropActive, setDropActive] = useState(false);
  const [contextNoteTitle, setContextNoteTitle] = useState("");
  const [contextNoteBody, setContextNoteBody] = useState("");
  const [contextNoteError, setContextNoteError] = useState<string | null>(null);
  const [revisionResource, setRevisionResource] = useState<ResourceView | null>(null);
  const createWorkspaceNote = () => {
    const title = contextNoteTitle.trim();
    const body = contextNoteBody;
    if (!selectedWorkspaceId || !title || !body.trim() || busy) return;
    if (new TextEncoder().encode(body).byteLength > 64 * 1024) {
      setContextNoteError("Workspace notes are limited to 64 KiB of UTF-8 text.");
      return;
    }
    setContextNoteError(null);
    const safeTitle = title.normalize("NFKC").replace(/[^\p{L}\p{N} _-]/gu, "-").replace(/\s+/gu, " ").replace(/^[-. ]+|[-. ]+$/gu, "").slice(0, 80) || "Workspace note";
    const file = new File([body], `${safeTitle}.md`, { type: "text/markdown", lastModified: 0 });
    void onFiles([file], {
      kind: "WORKSPACE_NOTES",
      owner_ref: { kind: "WORKSPACE", workspace_id: selectedWorkspaceId },
    });
  };
  useEffect(() => { folderInput.current?.setAttribute("webkitdirectory", ""); }, []);
  useEffect(() => {
    setQuery("");
    setSearchResults([]);
    setSearchCursor(null);
    setSearchError(null);
    setContentScanInfo(null);
    setSearchMode("METADATA");
    setSearchKind("");
    setSearchFreshness("");
    setIndexActionMessage(null);
    setIndexActionError(null);
  }, [selectedWorkspaceId]);
  useEffect(() => {
    const generation = ++rootListGeneration.current;
    setWorkspaceRoots([]);
    setRootNextCursor(null);
    setRootError(null);
    setRootMessage(null);
    setRootStatusAction(null);
    rootScanActionRef.current = null;
    setRootScanAction(null);
    setRootScanFeedback({});
    setRootBusy(false);
    if (!selectedWorkspaceId || !operatorReady) return;
    setRootBusy(true);
    void invoke<WorkspaceRootPageView>("list_workspace_roots", { workspaceId: selectedWorkspaceId, cursor: null })
      .then((page) => {
        if (generation !== rootListGeneration.current) return;
        setWorkspaceRoots(page.items);
        setRootNextCursor(page.nextCursor);
      })
      .catch((error) => {
        if (generation === rootListGeneration.current) {
          setRootError(typeof error === "string" ? error : error instanceof Error ? error.message : "Persistent folders could not be loaded.");
        }
      })
      .finally(() => { if (generation === rootListGeneration.current) setRootBusy(false); });
    return () => { rootListGeneration.current += 1; };
  }, [selectedWorkspaceId, operatorReady]);
  useEffect(() => {
    const normalizedQuery = query.trim();
    const request = ++searchRequest.current;
    const hasSearchFilters = Boolean(searchKind || searchFreshness);
    const metadataFilterOnly = searchMode === "METADATA" && hasSearchFilters;
    if ((!normalizedQuery && !metadataFilterOnly) || !selectedWorkspaceId) {
      setSearchResults([]);
      setSearchCursor(null);
      setSearchBusy(false);
      setSearchError(null);
      setContentScanInfo(null);
      return;
    }
    setSearchResults([]);
    setSearchCursor(null);
    setContentScanInfo(null);
    if (busy) {
      setSearchBusy(false);
      return;
    }
    const timer = window.setTimeout(() => {
      setSearchBusy(true);
      setSearchError(null);
      void invoke<ResourceSearchPageView>("search_resources", {
        workspaceId: selectedWorkspaceId,
        query: normalizedQuery,
        mode: searchMode,
        kind: searchKind || null,
        freshness: searchFreshness || null,
        cursor: null,
      }).then((page) => {
        if (isCurrentWorkspaceSearchRequest(request, searchRequest.current, selectedWorkspaceId, selectedWorkspaceIdRef.current)) {
          setSearchResults(page.items);
          setSearchCursor(page.nextCursor);
          setContentScanInfo(page.contentScan);
        }
      }).catch((error) => {
        if (isCurrentWorkspaceSearchRequest(request, searchRequest.current, selectedWorkspaceId, selectedWorkspaceIdRef.current)) {
          setSearchError(typeof error === "string" ? error : error instanceof Error ? error.message : "Workspace search is unavailable.");
          setSearchResults([]);
          setSearchCursor(null);
          setContentScanInfo(null);
        }
      }).finally(() => {
        if (isCurrentWorkspaceSearchRequest(request, searchRequest.current, selectedWorkspaceId, selectedWorkspaceIdRef.current)) setSearchBusy(false);
      });
    }, 220);
    return () => window.clearTimeout(timer);
  }, [query, selectedWorkspaceId, resources, busy, searchMode, searchKind, searchFreshness, resourceIndexRefreshNonce]);
  const rebuildResourceTextIndex = async (resource: ResourceView) => {
    if (!selectedWorkspaceId || !operatorReady || rebuildBusyResourceId) return;
    const workspaceId = selectedWorkspaceId;
    const pin = {
      workspaceId,
      resourceId: resource.resourceId,
      resourceRevisionId: resource.resourceRevisionId,
      contentDigest: resource.contentDigest,
    };
    setRebuildBusyResourceId(resource.resourceId);
    setIndexActionMessage(null);
    setIndexActionError(null);
    try {
      const outcome = await performResourceIndexRebuild({
        requests: resourceIndexRequestKeys.current,
        pin,
        createRequestId: () => crypto.randomUUID(),
        execute: (requestId, exactPin) => invoke<ResourceTextIndexRebuildView>("rebuild_resource_text_index", {
          workspaceId: exactPin.workspaceId,
          resourceId: exactPin.resourceId,
          resourceRevisionId: exactPin.resourceRevisionId,
          contentDigest: exactPin.contentDigest,
          requestId,
        }),
      });
      if (selectedWorkspaceIdRef.current !== workspaceId) return;
      if (outcome.kind === "failed") {
        const retryPin = outcome.retryPin ? {
          workspaceId: outcome.retryPin.workspaceId,
          resourceId: outcome.retryPin.resourceId,
          revisionId: outcome.retryPin.resourceRevisionId,
          contentDigest: outcome.retryPin.contentDigest,
        } : null;
        setIndexRetryPin((current) => current?.workspaceId === pin.workspaceId
          && current.resourceId === pin.resourceId
          && current.revisionId === pin.resourceRevisionId
          && current.contentDigest === pin.contentDigest ? retryPin : retryPin ?? current);
        setIndexActionError(outcome.message);
        return;
      }
      const result = outcome.receipt;
      setIndexRetryPin((current) => current?.workspaceId === pin.workspaceId
        && current.resourceId === pin.resourceId
        && current.revisionId === pin.resourceRevisionId
        && current.contentDigest === pin.contentDigest ? null : current);
      setResourceIndexRefreshNonce((current) => current + 1);
      if (result.outcome === "INDEXED") {
        setIndexActionMessage(`Encrypted local text index rebuilt for “${resource.displayName}”.`);
      } else {
        const reason = result.reason === "OVER_SIZE_LIMIT" ? "it is over the 1 MiB limit"
          : result.reason === "INVALID_UTF8" ? "it is not valid UTF-8 text"
            : result.reason === "CONTROL_CHARACTERS" ? "it contains unsupported control characters"
              : result.reason === "TERM_LIMIT_EXCEEDED" ? "it exceeds the local term limit"
                : "this file type is not supported";
        setIndexActionMessage(`“${resource.displayName}” was not indexed because ${reason}.`);
      }
    } finally {
      setRebuildBusyResourceId(null);
    }
  };
  const saveResourceAs = async (resource: ResourceView) => {
    if (!selectedWorkspaceId || !operatorReady || saveAsBusyResourceId) return;
    setSaveAsBusyResourceId(resource.resourceId);
    setSaveAsMessage(null);
    setSaveAsError(null);
    try {
      const result = await invoke<{ status: string }>("resource_save_as", {
        workspaceId: selectedWorkspaceId,
        resourceId: resource.resourceId,
        expectedResourceRevisionId: resource.resourceRevisionId,
        expectedContentDigest: resource.contentDigest,
        expectedSizeBytes: resource.sizeBytes,
        expectedMediaType: resource.mediaType,
      });
      if (!result || Object.keys(result).join(",") !== "status" || !["SAVED", "CANCELLED"].includes(result.status)) {
        throw new Error("Resource Save As response is invalid.");
      }
      if (result.status === "SAVED") setSaveAsMessage(`${resource.displayName} was saved to the selected location.`);
    } catch (error) {
      setSaveAsError(typeof error === "string" ? error : error instanceof Error ? error.message : "Resource could not be saved.");
    } finally {
      setSaveAsBusyResourceId(null);
    }
  };
  const loadMoreSearchResults = async () => {
    const normalizedQuery = query.trim();
    const workspaceId = selectedWorkspaceId;
    const cursor = searchCursor;
    const request = searchRequest.current;
    const metadataFilterOnly = searchMode === "METADATA" && Boolean(searchKind || searchFreshness);
    if ((!normalizedQuery && !metadataFilterOnly) || !workspaceId || selectedWorkspaceIdRef.current !== workspaceId || !cursor || searchBusy) return;
    setSearchBusy(true);
    setSearchError(null);
    try {
      const page = await invoke<ResourceSearchPageView>("search_resources", {
        workspaceId,
        query: normalizedQuery,
        mode: searchMode,
        kind: searchKind || null,
        freshness: searchFreshness || null,
        cursor,
      });
      if (!isCurrentWorkspaceSearchRequest(request, searchRequest.current, workspaceId, selectedWorkspaceIdRef.current)) return;
      const seen = new Set(searchResults.map((item) => item.resourceId));
      if (page.items.some((item) => seen.has(item.resourceId)) || page.nextCursor === cursor) throw new Error("Workspace search returned a repeated page or cursor.");
      setSearchResults((current) => [...current, ...page.items.filter((item) => !current.some((existing) => existing.resourceId === item.resourceId))]);
      setSearchCursor(page.nextCursor);
      setContentScanInfo(page.contentScan);
    } catch (error) {
      if (isCurrentWorkspaceSearchRequest(request, searchRequest.current, workspaceId, selectedWorkspaceIdRef.current)) setSearchError(typeof error === "string" ? error : error instanceof Error ? error.message : "More search results could not be loaded.");
    } finally {
      if (isCurrentWorkspaceSearchRequest(request, searchRequest.current, workspaceId, selectedWorkspaceIdRef.current)) setSearchBusy(false);
    }
  };
  const addPersistentFolder = async () => {
    if (!selectedWorkspaceId || workspaceVersion === null || rootAdding || busy) return;
    const generation = rootListGeneration.current;
    const request = {
      action: "add" as const,
      workspaceId: selectedWorkspaceId,
      expectedVersion: workspaceVersion,
      watchPolicy: "METADATA",
      replicationPolicy: "NONE",
    };
    const mutation = workspaceRootMutationRequest(rootStatusRequestKeys.current, request, () => crypto.randomUUID());
    setRootAdding(true);
    setRootError(null);
    setRootMessage(null);
    try {
      const root = await invoke<WorkspaceRootView | null>("add_workspace_root", {
        workspaceId: selectedWorkspaceId,
        expectedWorkspaceVersion: workspaceVersion,
        watchPolicy: "METADATA",
        replicationPolicy: "NONE",
        requestId: mutation.requestId,
      });
      if (!root) {
        clearWorkspaceRootMutationRequest(rootStatusRequestKeys.current, mutation.key);
        return;
      }
      if (!isWorkspaceRootAddResponse(root, request)) {
        throw new Error("The Runtime returned an invalid persistent folder receipt.");
      }
      clearWorkspaceRootMutationRequest(rootStatusRequestKeys.current, mutation.key);
      if (generation !== rootListGeneration.current) return;
      setWorkspaceRoots((current) => [root, ...current.filter((item) => item.workspaceRootId !== root.workspaceRootId)]);
      setRootMessage(`“${root.displayName}” was saved as a persistent folder scope. Folder watching and indexing are not active in this build.`);
    } catch (error) {
      if (generation === rootListGeneration.current) setRootError(typeof error === "string" ? error : error instanceof Error ? error.message : "The persistent folder could not be added.");
    } finally {
      setRootAdding(false);
    }
  };
  const loadMoreRoots = async () => {
    const cursor = rootNextCursor;
    const generation = rootListGeneration.current;
    if (!selectedWorkspaceId || !cursor || rootBusy) return;
    setRootBusy(true);
    setRootError(null);
    try {
      const page = await invoke<WorkspaceRootPageView>("list_workspace_roots", { workspaceId: selectedWorkspaceId, cursor });
      if (generation !== rootListGeneration.current) return;
      setWorkspaceRoots((current) => [...current, ...page.items.filter((item) => !current.some((existing) => existing.workspaceRootId === item.workspaceRootId))]);
      setRootNextCursor(page.nextCursor);
    } catch (error) {
      if (generation === rootListGeneration.current) setRootError(typeof error === "string" ? error : error instanceof Error ? error.message : "More persistent folders could not be loaded.");
    } finally {
      if (generation === rootListGeneration.current) setRootBusy(false);
    }
  };
  const revokePersistentFolder = async (root: WorkspaceRootView) => {
    if (!selectedWorkspaceId || root.workspaceId !== selectedWorkspaceId || rootRevokingId) return;
    const confirmed = window.confirm(`Remove LiteCowork's saved access to “${root.displayName}”? Existing replicated copies are not deleted.`);
    if (!confirmed) return;
    const generation = rootListGeneration.current;
    const request = {
      action: "revoke" as const,
      workspaceId: selectedWorkspaceId,
      workspaceRootId: root.workspaceRootId,
      expectedVersion: root.version,
    };
    const mutation = workspaceRootMutationRequest(rootStatusRequestKeys.current, request, () => crypto.randomUUID());
    setRootRevokingId(root.workspaceRootId);
    setRootError(null);
    setRootMessage(null);
    try {
      const revoked = await invoke<WorkspaceRootView>("revoke_workspace_root", {
        workspaceId: selectedWorkspaceId,
        workspaceRootId: root.workspaceRootId,
        expectedVersion: root.version,
        requestId: mutation.requestId,
      });
      if (!isWorkspaceRootRevokeResponse(revoked, request)) {
        throw new Error("The Runtime returned an invalid folder removal receipt.");
      }
      clearWorkspaceRootMutationRequest(rootStatusRequestKeys.current, mutation.key);
      if (generation !== rootListGeneration.current) return;
      setWorkspaceRoots((current) => current.map((item) => item.workspaceRootId === revoked.workspaceRootId ? revoked : item));
      setRootMessage(`Access to “${revoked.displayName}” was revoked. Existing replicated copies are unchanged.`);
    } catch (error) {
      if (generation === rootListGeneration.current) setRootError(typeof error === "string" ? error : error instanceof Error ? error.message : "Persistent folder access could not be removed.");
    } finally {
      setRootRevokingId(null);
    }
  };
  const updatePersistentFolderStatus = async (root: WorkspaceRootView, action: "pause" | "resume") => {
    const expectedStatus = action === "pause" ? "ACTIVE" : "PAUSED";
    if (!selectedWorkspaceId || root.workspaceId !== selectedWorkspaceId || root.status !== expectedStatus
      || rootStatusAction || rootRevokingId || rootScanActionRef.current) return;
    if (!Number.isSafeInteger(root.version) || root.version < 1 || root.version >= Number.MAX_SAFE_INTEGER) {
      setRootError("This folder version cannot be safely updated. Reload the Library before trying again.");
      return;
    }
    const generation = rootListGeneration.current;
    const request = {
      workspaceId: selectedWorkspaceId,
      workspaceRootId: root.workspaceRootId,
      action,
      expectedVersion: root.version,
    } as const;
    const isCurrentRequest = () => isCurrentWorkspaceRootRequest(
      selectedWorkspaceId,
      generation,
      selectedWorkspaceIdRef.current,
      rootListGeneration.current,
    );
    setRootStatusAction({ workspaceRootId: root.workspaceRootId, action });
    setRootError(null);
    setRootMessage(null);
    try {
      const outcome = await performWorkspaceRootStatusAction({
        requests: rootStatusRequestKeys.current,
        request,
        createRequestId: () => crypto.randomUUID(),
        execute: (requestId) => invoke<WorkspaceRootView>(action === "pause" ? "pause_workspace_root" : "resume_workspace_root", {
          workspaceId: selectedWorkspaceId,
          workspaceRootId: root.workspaceRootId,
          expectedVersion: root.version,
          requestId,
        }),
        list: (cursor) => invoke<WorkspaceRootPageView>("list_workspace_roots", { workspaceId: selectedWorkspaceId, cursor }),
        isCurrent: isCurrentRequest,
        onRefreshStart: () => setRootBusy(true),
      });
      if (outcome.kind === "stale" || !isCurrentRequest()) return;
      if (outcome.kind === "confirmed") {
        setWorkspaceRoots((current) => current.map((item) => item.workspaceRootId === outcome.root.workspaceRootId ? outcome.root : item));
        setRootMessage(action === "pause"
          ? `“${outcome.root.displayName}” is paused. Its saved permission remains; folder watching and indexing are not active in this build.`
          : `“${outcome.root.displayName}” is active again. Folder watching and indexing are not active in this build.`);
      } else {
        if (outcome.refreshedRoots.length > 0) {
          const refreshedById = new Map(outcome.refreshedRoots.map((item) => [item.workspaceRootId, item]));
          setWorkspaceRoots((current) => current.map((item) => refreshedById.get(item.workspaceRootId) ?? item));
        }
        const refreshStatus = outcome.refreshError
          ? ` The latest folder status could not be confirmed: ${outcome.refreshError}`
          : outcome.targetFound
            ? " The folder list was reloaded; check its saved status before trying again."
            : " The folder list was reloaded, but the affected folder was not found; its previous row is retained.";
        setRootError(`${outcome.failureMessage}${refreshStatus}`);
        setRootBusy(false);
      }
    } finally {
      if (isCurrentRequest()) setRootStatusAction(null);
    }
  };
  const scanPersistentFolder = async (root: WorkspaceRootView) => {
    const workspaceId = selectedWorkspaceIdRef.current;
    if (!workspaceId || !operatorReady || rootBusy || rootStatusAction || rootRevokingId || rootScanActionRef.current || busy) return;
    const currentRoot = () => workspaceRootsRef.current.find((item) =>
      item.workspaceId === workspaceId && item.workspaceRootId === root.workspaceRootId,
    ) ?? null;
    const currentBeforeRequest = currentRoot();
    if (!currentBeforeRequest || currentBeforeRequest.version !== root.version
      || !isWorkspaceRootScannable(root, workspaceId)
      || !isWorkspaceRootScannable(currentBeforeRequest, workspaceId)) return;

    const requestGeneration = rootListGeneration.current;
    const request = workspaceRootScanRequest(rootScanRequests.current, {
      workspaceId,
      workspaceRootId: root.workspaceRootId,
      expectedVersion: root.version,
    }, () => crypto.randomUUID());
    const feedbackKey = JSON.stringify([workspaceId, root.workspaceRootId]);
    const action = { request, feedbackKey };
    const selectionStillCurrent = () => isCurrentWorkspaceRootRequest(
      request.workspaceId,
      requestGeneration,
      selectedWorkspaceIdRef.current,
      rootListGeneration.current,
    );
    rootScanActionRef.current = action;
    setRootScanAction(action);
    setRootScanFeedback((current) => ({
      ...current,
      [feedbackKey]: { kind: "loading", message: "Scanning folder metadata…", rootVersion: request.expectedVersion },
    }));

    try {
      const response = await invoke<unknown>("scan_workspace_root", {
        workspaceId: request.workspaceId,
        workspaceRootId: request.workspaceRootId,
        expectedVersion: request.expectedVersion,
        requestId: request.requestId,
      });
      if (!acceptWorkspaceRootScanResponse(rootScanRequests.current, request, response)) {
        throw new Error("The Runtime returned an invalid folder scan summary.");
      }
      if (!selectionStillCurrent()) return;
      const latestRoot = currentRoot();
      if (!isCurrentWorkspaceRootScan(
        request,
        requestGeneration,
        selectedWorkspaceIdRef.current,
        rootListGeneration.current,
        latestRoot,
      )) {
        if (latestRoot) {
          setRootScanFeedback((current) => ({
            ...current,
            [feedbackKey]: {
              kind: "error",
              message: "The folder changed while its scan was running. The scan result was not shown; review the folder status before scanning again.",
              rootVersion: latestRoot.version,
            },
          }));
        }
        return;
      }
      setRootScanFeedback((current) => ({
        ...current,
        [feedbackKey]: { kind: "result", message: workspaceRootScanAnnouncement(response), rootVersion: request.expectedVersion },
      }));
    } catch (error) {
      if (!selectionStillCurrent()) return;
      const latestRoot = currentRoot();
      const failure = workspaceRootScanFailure(error, request, latestRoot);
      setRootScanFeedback((current) => ({
        ...current,
        [feedbackKey]: {
          kind: failure.kind === "stale"
            ? "stale"
            : failure.kind === "unsupported"
              ? "unsupported"
              : failure.retrySameRequest
                ? "error"
                : "rejected",
          message: failure.message,
          rootVersion: latestRoot?.version ?? request.expectedVersion,
        },
      }));
    } finally {
      if (rootScanActionRef.current?.request.requestId === request.requestId) {
        rootScanActionRef.current = null;
        setRootScanAction(null);
      }
    }
  };
  const hasSearchFilters = Boolean(searchKind || searchFreshness);
  const searching = query.trim().length > 0 || hasSearchFilters;
  const searchPageBusy = searchBusy;
  return (
    <div className="page-content subpage-content">
      <div className="eyebrow">WORKSPACE FILES</div>
      <div className="section-heading library-heading"><div><h1>Library</h1><p>Store and find files in this Workspace. Adding a file does not attach it to a Task.</p></div></div>
      <ZipIntakeNotice workspaceId={selectedWorkspaceId} operatorReady={operatorReady} />
      {selectedTaskInputs.length > 0 && <section className="task-input-selection" aria-label="Files selected for a Task">
        <div><strong>{selectedTaskInputs.length} file{selectedTaskInputs.length === 1 ? "" : "s"} selected for a Task</strong><p>Each selection pins the exact Resource revision. The files are not read by an agent until Task execution is available.</p></div>
        <div className="task-input-selection-actions"><button type="button" className="quiet-button" onClick={onClearTaskInputs}>Clear</button><button type="button" className="primary-button" onClick={onUseSelectedInTask}>Use in Task</button></div>
      </section>}
      {!selectedWorkspaceId && <p className="inline-error" role="status">Select or create a Workspace before adding files.</p>}
      <section className="persistent-folders" aria-label="Persistent folder scopes">
        <div className="section-heading"><div><h2>Persistent folders</h2><p>Register a selected folder as a persistent scope. A manual scan counts folder metadata only: it does not add files to the Library, read file content, watch for changes, or infer deletions.</p></div><button className="quiet-button" type="button" disabled={!selectedWorkspaceId || !operatorReady || workspaceVersion === null || rootAdding || busy} onClick={() => void addPersistentFolder()}>{rootAdding ? "Opening folder picker…" : "Add persistent folder"}</button></div>
        {!operatorReady && selectedWorkspaceId && <p className="inline-status" role="status">Start the local Runtime to manage persistent folders.</p>}
        {rootBusy && workspaceRoots.length === 0 && <p className="inline-status" role="status">Loading persistent folders…</p>}
        {rootBusy && workspaceRoots.length > 0 && <p className="inline-status" role="status">Refreshing persistent folder status…</p>}
        {rootError && <p className="inline-error" role="status">{rootError}</p>}
        {rootMessage && <p className="inline-status" role="status" aria-live="polite">{rootMessage}</p>}
        {workspaceRoots.length > 0 && <ul className="resource-list persistent-folder-list">{workspaceRoots.map((root) => {
          const actionPending = rootStatusAction?.workspaceRootId === root.workspaceRootId;
          const scanFeedbackKey = JSON.stringify([root.workspaceId, root.workspaceRootId]);
          const scanFeedback = rootScanFeedback[scanFeedbackKey];
          const currentScanFeedback = scanFeedback?.rootVersion === root.version ? scanFeedback : null;
          const scanPending = rootScanAction?.feedbackKey === scanFeedbackKey;
          const scanRequiresRefresh = currentScanFeedback?.kind === "stale" || currentScanFeedback?.kind === "rejected";
          const scanUnavailable = currentScanFeedback?.kind === "unsupported";
          const feedbackIsAlert = currentScanFeedback?.kind === "error"
            || currentScanFeedback?.kind === "stale"
            || currentScanFeedback?.kind === "unsupported"
            || currentScanFeedback?.kind === "rejected";
          const canScanRoot = !scanUnavailable
            && root.workspaceId === selectedWorkspaceId
            && isWorkspaceRootScannable(root, selectedWorkspaceId);
          const controlsDisabled = !operatorReady || rootBusy || rootStatusAction !== null || rootRevokingId !== null || rootScanAction !== null;
          return <li key={root.workspaceRootId}>
            <span className="resource-file-icon" aria-hidden="true">▱</span>
            <span>
              <strong>{root.displayName}</strong>
              <small>{workspaceRootStatusLabel(root.status)}</small>
              <small>{resourceLocationAvailabilityLabel(root.locationAvailability)}</small>
              {currentScanFeedback && <small
                className={feedbackIsAlert ? "inline-error" : "inline-status"}
                role={feedbackIsAlert ? "alert" : "status"}
                aria-live={feedbackIsAlert ? "assertive" : "polite"}
              >{currentScanFeedback.message}</small>}
              {(root.status === "ACTIVE" || root.status === "PAUSED") && <WorkspaceRootObservations
                root={root}
                scanUnsupported={scanUnavailable}
                openResource={(resourceId, displayName, sizeBytes) => {
                  const catalogResource = resources.find((resource) => resource.resourceId === resourceId);
                  setRevisionResource(catalogResource ?? {
                    resourceId,
                    workspaceId: selectedWorkspaceId,
                    resourceRevisionId: "",
                    displayName,
                    mediaType: "application/octet-stream",
                    contentDigest: "",
                    sizeBytes,
                  });
                }}
                loadPage={(request: WorkspaceRootObservationRequest) => invoke<WorkspaceRootObservationPage>("search_workspace_root_observations", {
                  workspaceId: request.workspaceId,
                  workspaceRootId: request.workspaceRootId,
                  rootVersion: request.rootVersion,
                  query: request.query,
                  freshness: request.freshness || null,
                  cursor: request.cursor,
                  limit: request.limit,
                })}
              />}
            </span>
            <span className="resource-preview-unavailable">Metadata policy · inactive</span>
            <span className="persistent-folder-actions">
              {canScanRoot && <button className="quiet-button" type="button"
                aria-label={`Scan folder metadata in ${root.displayName}`}
                disabled={controlsDisabled || busy || scanRequiresRefresh}
                onClick={() => void scanPersistentFolder(root)}
              >{scanPending ? "Scanning…" : "Scan folder"}</button>}
              {root.status === "ACTIVE" && <button className="quiet-button" type="button" aria-label={`Pause ${root.displayName}`} disabled={controlsDisabled} onClick={() => void updatePersistentFolderStatus(root, "pause")}>
                {actionPending && rootStatusAction?.action === "pause" ? "Pausing…" : "Pause"}
              </button>}
              {root.status === "PAUSED" && <button className="quiet-button" type="button" aria-label={`Resume ${root.displayName}`} disabled={controlsDisabled} onClick={() => void updatePersistentFolderStatus(root, "resume")}>
                {actionPending && rootStatusAction?.action === "resume" ? "Checking folder…" : "Revalidate & resume"}
              </button>}
              {root.status !== "REVOKED" && <button className="quiet-button" type="button" aria-label={`Remove access to ${root.displayName}`} disabled={controlsDisabled} onClick={() => void revokePersistentFolder(root)}>
                {rootRevokingId === root.workspaceRootId ? "Removing…" : "Remove access"}
              </button>}
            </span>
          </li>;
        })}</ul>}
        {rootNextCursor && <div className="load-more-row"><button className="quiet-button" type="button" disabled={rootBusy} onClick={() => void loadMoreRoots()}>{rootBusy ? "Loading…" : "Load more folders"}</button></div>}
      </section>
      <section
        className={`resource-intake${dropActive ? " drop-active" : ""}${!selectedWorkspaceId ? " intake-disabled" : ""}`}
        aria-label="Add files to this Workspace"
        onDragEnter={(event) => { event.preventDefault(); if (selectedWorkspaceId && !busy && Array.from(event.dataTransfer.types).includes("Files")) setDropActive(true); }}
        onDragOver={(event) => { event.preventDefault(); if (selectedWorkspaceId && !busy && Array.from(event.dataTransfer.types).includes("Files")) event.dataTransfer.dropEffect = "copy"; }}
        onDragLeave={(event) => { if (!event.currentTarget.contains(event.relatedTarget as Node | null)) setDropActive(false); }}
        onDrop={(event) => { event.preventDefault(); setDropActive(false); if (selectedWorkspaceId && !busy && event.dataTransfer.files.length > 0) void onFiles(event.dataTransfer.files); }}
      >
        <div className="resource-intake-copy">
          <span className="resource-intake-icon" aria-hidden="true">＋</span>
          <div>
            <h2>{dropActive ? "Drop files to add them" : busy ? "Adding files to this Workspace…" : "Add files to this Workspace"}</h2>
            <p>Choose files, a folder, or a ZIP archive. You can also drag files here.</p>
          </div>
        </div>
        <div className="resource-intake-actions">
          {busy && <button className="quiet-button" type="button" disabled={pausePending} onClick={onPauseImport}>{pausePending ? "Pausing…" : "Pause upload"}</button>}
          <label className="quiet-button file-picker-button">
            Add folder
            <input ref={folderInput} type="file" multiple disabled={busy || !selectedWorkspaceId} onChange={(event) => { void onFiles(event.currentTarget.files); event.currentTarget.value = ""; }} />
          </label>
          <label className="primary-button file-picker-button">
            {busy ? "Adding…" : "Choose files"}
            <input type="file" multiple disabled={busy || !selectedWorkspaceId} accept=".zip,.pdf,.txt,.md,.csv,.json,.docx,.xlsx,.pptx,image/*" onChange={(event) => { void onFiles(event.currentTarget.files); event.currentTarget.value = ""; }} />
          </label>
        </div>
        <ul className="resource-intake-facts">
          <li>One-time copy of the selected files; a folder is not watched for future changes.</li>
          <li>ZIP files are saved intact. Their contents are not unpacked.</li>
          <li>Up to 100 files and 100 MiB per selection; likely secret and generated files are skipped.</li>
        </ul>
        <div className="context-note-create" aria-label="Create a Workspace note">
          <h3>Write a Workspace note</h3>
          <p>This creates a Workspace-scoped text Resource with revision history. It is not automatically sent to agents and does not enable semantic RAG.</p>
          <label htmlFor="workspace-note-title">Title</label>
          <input id="workspace-note-title" type="text" maxLength={80} value={contextNoteTitle} onChange={(event) => { setContextNoteTitle(event.currentTarget.value); setContextNoteError(null); }} placeholder="Project conventions" disabled={busy || !selectedWorkspaceId} />
          <label htmlFor="workspace-note-body">Note</label>
          <textarea id="workspace-note-body" value={contextNoteBody} onChange={(event) => { setContextNoteBody(event.currentTarget.value); setContextNoteError(null); }} maxLength={64 * 1024} rows={4} placeholder="Write a short note for this Workspace…" disabled={busy || !selectedWorkspaceId} />
          <div className="context-note-footer"><small>Stored only in this Workspace. Maximum 64 KiB of UTF-8 text.</small><button className="quiet-button" type="button" disabled={!operatorReady || busy || !contextNoteTitle.trim() || !contextNoteBody.trim()} onClick={createWorkspaceNote}>Save Workspace note</button></div>
          {contextNoteError && <p className="inline-error" role="alert">{contextNoteError}</p>}
        </div>
      </section>
      {message && <p className="inline-status resource-intake-status" role="status" aria-live="polite">{message}</p>}
      {uploadResumeForget?.workspaceId === selectedWorkspaceId && <section className="resource-upload-recovery" aria-label="Upload recovery needs your review">
        <div>
          <h3>Saved upload requires your review</h3>
          <p>This upload is {uploadResumeForget.terminalState.toLowerCase()}. Review the Resource list before starting another upload.</p>
          <small>Forgetting clears only this app’s saved retry identity. It does not delete or change the server-side upload session.</small>
        </div>
        <button className="quiet-button" type="button" disabled={busy} onClick={onForgetUploadResume}>Forget upload resume details</button>
      </section>}
      {indexActionMessage && <p className="inline-status" role="status" aria-live="polite">{indexActionMessage}</p>}
      {indexActionError && <p className="inline-error" role="alert">{indexActionError}{indexRetryPin?.workspaceId === selectedWorkspaceId ? " Retry stays pinned to the same Resource revision in this desktop session." : ""}</p>}
      {saveAsMessage && <p className="inline-status" role="status" aria-live="polite">{saveAsMessage}</p>}
      {saveAsError && <p className="inline-error" role="alert">{saveAsError}</p>}
      {indexActionError?.includes("Reload the Library") && <button className="quiet-button" type="button" disabled={!operatorReady} onClick={() => { setIndexActionError(null); onRefreshResources(); }}>Reload files</button>}
      {resources.length === 0 && listFailed ? <div className="empty-panel subpage-panel" role="alert"><div className="empty-icon" aria-hidden="true">▧</div><h2>Saved Resources are unavailable</h2><p>The last read of this Workspace's file list failed. Nothing is known about whether this Workspace has files, so this is not an empty result.</p><button type="button" className="quiet-button" disabled={!operatorReady} onClick={onRefreshResources}>Retry file list</button></div> : resources.length === 0 ? <div className="empty-panel subpage-panel"><div className="empty-icon" aria-hidden="true">▧</div><h2>No files in this Workspace yet</h2><p>After adding files, find them by name or type. Small supported text files can also be scanned on demand; ZIP contents are not extracted.</p></div> : (
        <>
          <div className="library-search-block">
            <label htmlFor="resource-search">Find a saved file</label>
            <input id="resource-search" className="resource-search" type="search" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search by file name, type, or text" />
            <label className="resource-content-search-toggle" htmlFor="resource-search-mode">Search in</label>
            <select id="resource-search-mode" className="resource-search-mode" value={searchMode} onChange={(event) => setSearchMode(event.currentTarget.value as typeof searchMode)}>
              <option value="METADATA">Names and file types</option>
              <option value="INDEXED_CONTENT">Indexed local text</option>
              <option value="ON_DEMAND_CONTENT">Scan small text files now</option>
            </select>
            <div className="resource-search-filters" aria-label="Resource filters">
              <label htmlFor="resource-kind-filter">File type</label>
              <select id="resource-kind-filter" className="resource-search-mode" value={searchKind} onChange={(event) => setSearchKind(event.currentTarget.value)}>
                <option value="">All types</option>
                <option value="FILE">Files</option>
                <option value="FOLDER">Folders</option>
                <option value="ARTIFACT">Artifacts</option>
                <option value="CONNECTOR_OBJECT">Connected items</option>
                <option value="WEB_RESOURCE">Web resources</option>
                <option value="OTHER">Other</option>
              </select>
              <label htmlFor="resource-freshness-filter">Freshness</label>
              <select id="resource-freshness-filter" className="resource-search-mode" value={searchFreshness} onChange={(event) => setSearchFreshness(event.currentTarget.value)}>
                <option value="">Any freshness</option>
                <option value="CURRENT">Current</option>
                <option value="STALE">Stale</option>
                <option value="UNKNOWN">Unknown</option>
                <option value="UNAVAILABLE">Unavailable</option>
              </select>
            </div>
            <p className="resource-search-hint">{searchMode === "INDEXED_CONTENT"
              ? "Searches the encrypted local index for current plain-text Resources up to 1 MiB. All words must match. ZIP, PDF, Office files, folders, OCR, and semantic search are not included."
              : searchMode === "ON_DEMAND_CONTENT"
                ? "Reads up to 20 current managed Resources per request, with a 1 MiB per-file and 8 MiB total limit. Extracted text is not saved. ZIP, PDF, Office files, and folders are not scanned."
                : "Search checks saved names and media types. Text contents are not read."}</p>
            {searchMode === "ON_DEMAND_CONTENT" && contentScanInfo && <p className="resource-search-hint" role="status">Checked {contentScanInfo.textResourcesChecked} text files ({contentScanInfo.candidatesScanned} Resources considered). {contentScanInfo.skippedUnsupportedType > 0 ? `${contentScanInfo.skippedUnsupportedType} unsupported type(s); ` : ""}{contentScanInfo.skippedOverFileLimit > 0 ? `${contentScanInfo.skippedOverFileLimit} over the 1 MiB limit; ` : ""}{contentScanInfo.skippedRevisionChanged > 0 ? `${contentScanInfo.skippedRevisionChanged} changed during search; ` : ""}{contentScanInfo.byteBudgetExhausted || contentScanInfo.candidateBudgetExhausted ? "Load more to continue." : "No persistent text index is used."}</p>}
          </div>
          {searching ? <>
            {searchError && <p className="inline-error" role="status">{searchError}</p>}
            {searchPageBusy && searchResults.length === 0 ? <p className="inline-status" role="status">{searchMode === "ON_DEMAND_CONTENT" ? "Scanning bounded local text…" : searchMode === "INDEXED_CONTENT" ? "Searching indexed local text…" : "Searching this Workspace…"}</p> : searchResults.length === 0 && !searchError ? <p className="empty-inline">{query.trim() ? `No saved files match “${query.trim()}” in the checked results.` : searchMode === "METADATA" ? "No saved files match the selected filters." : "Enter search terms to apply filters to content search."}</p> : <ul className="resource-list">{searchResults.map((result) => {
              const attached = selectedTaskInputs.find((input) => input.resourceId === result.resourceId);
              const exactRevisionAttached = attached?.revisionId === result.resourceRevisionId;
              return <li key={result.resourceId}><span className="resource-file-icon" aria-hidden="true">▤</span><span><strong>{result.displayName}</strong><small>{result.matchReasons.map((reason) => reason === "MEDIA_TYPE" ? "Type match" : reason === "CONTENT_ON_DEMAND" ? "Text match · scanned now" : reason === "CONTENT_INDEXED" ? "Text match · local index" : "Name match").join(" · ") || "Saved Resource"} · {result.freshness.toLocaleLowerCase()}</small></span><span className={result.snippet ? "resource-search-snippet" : "resource-digest"}>{result.snippet ?? "Metadata match"}</span><button className="quiet-button" type="button" onClick={() => void onPreview(result.resourceId, result.displayName, result.resourceRevisionId, result.sourceMatches.length > 0 ? { contentDigest: result.sourceContentDigest, matches: result.sourceMatches } : undefined)} aria-expanded={previewResourceId === result.resourceId}>Preview</button><button className="quiet-button" type="button" aria-pressed={exactRevisionAttached} onClick={() => onToggleTaskInput({ resourceId: result.resourceId, resourceRevisionId: result.resourceRevisionId, displayName: result.displayName })}>{exactRevisionAttached ? "Remove input" : attached ? "Use this revision" : "Add to Task"}</button></li>;
            })}</ul>}
            {selectedWorkspaceId && <ResourceSourceReferences
              workspaceId={selectedWorkspaceId}
              references={resourceSearchSourceReferences(selectedWorkspaceId, searchResults)}
              onOpenExactSource={(pin) => {
                const source = searchResults.find((result) => result.resourceId === pin.resourceId && result.resourceRevisionId === pin.revisionId);
                if (!source) return;
                void onPreview(source.resourceId, source.displayName, pin.revisionId, {
                  contentDigest: pin.contentDigest,
                  matches: [...pin.sourceMatches],
                });
              }}
            />}
            {searchCursor && <div className="load-more-row"><button className="quiet-button" type="button" disabled={searchBusy} onClick={() => void loadMoreSearchResults()}>{searchBusy ? "Searching…" : "Load more results"}</button></div>}
          </> : <>
            {resources.length === 0 ? null : <ul className="resource-list">{resources.map((resource) => {
              const previewable = supportsTextPreview(resource.mediaType, resource.sizeBytes);
              const indexAction = resourceIndexAction(resource);
              const isZip = indexAction.kind === "UNAVAILABLE";
              const attached = selectedTaskInputs.find((input) => input.resourceId === resource.resourceId);
              const exactRevisionAttached = attached?.revisionId === resource.resourceRevisionId;
              const canRetryIndex = indexRetryPin?.workspaceId === selectedWorkspaceId
                && indexRetryPin.resourceId === resource.resourceId
                && indexRetryPin.revisionId === resource.resourceRevisionId
                && indexRetryPin.contentDigest === resource.contentDigest;
              return <li key={resource.resourceId}><span className="resource-file-icon" aria-hidden="true">▤</span><span><strong>{resource.displayName}</strong><small>{formatBytes(resource.sizeBytes)} · {isZip ? "ZIP archive stored intact" : resource.mediaType}</small></span><span className="resource-digest">{resource.contentDigest.slice(0, 19)}…</span>{previewable ? <button className="quiet-button" type="button" onClick={() => void onPreview(resource.resourceId, resource.displayName, resource.resourceRevisionId)} aria-expanded={previewResourceId === resource.resourceId}>Preview text</button> : <span className="resource-preview-unavailable" title={resource.sizeBytes > 1024 * 1024 ? "Text preview is limited to 1 MiB." : isZip ? "ZIP files are stored intact and are not extracted." : "Preview is available for text files only."}>{resource.sizeBytes > 1024 * 1024 ? "Over preview limit" : isZip ? "Not extracted" : "No text preview"}</span>}{resource.sizeBytes <= 10 * 1024 * 1024 ? <button className="quiet-button" type="button" disabled={!operatorReady || saveAsBusyResourceId !== null || busy} onClick={() => void saveResourceAs(resource)}>{saveAsBusyResourceId === resource.resourceId ? "Saving…" : "Save original…"}</button> : <span className="resource-preview-unavailable" title="Desktop Save As is limited to 10 MiB.">Over Save As limit</span>}{indexAction.kind === "REBUILD" ? <button className="quiet-button" type="button" disabled={!operatorReady || rebuildBusyResourceId !== null || busy} onClick={() => void rebuildResourceTextIndex(resource)}>{rebuildBusyResourceId === resource.resourceId ? "Rebuilding index…" : canRetryIndex ? "Retry index rebuild" : indexAction.label}</button> : <span className="resource-preview-unavailable" title={indexAction.reason}>{indexAction.label}</span>}<button className="quiet-button" type="button" onClick={() => setRevisionResource(resource)} aria-expanded={revisionResource?.resourceId === resource.resourceId}>History &amp; edit</button><button className="quiet-button" type="button" aria-pressed={exactRevisionAttached} onClick={() => onToggleTaskInput({ resourceId: resource.resourceId, resourceRevisionId: resource.resourceRevisionId, displayName: resource.displayName })}>{exactRevisionAttached ? "Remove input" : attached ? "Use this revision" : "Add to Task"}</button></li>;
            })}</ul>}
            {nextCursor && <div className="load-more-row"><button className="quiet-button" type="button" disabled={pageBusy} onClick={() => void onLoadMore()}>{pageBusy ? "Loading…" : "Load older files"}</button></div>}
          </>}
          {previewResourceId && <section className="resource-preview" aria-label="Resource text preview">
            <div className="section-heading"><h2>{previewResourceName ?? "Preview"}</h2><button className="quiet-button" type="button" onClick={onClosePreview}>Close preview</button></div>
            {previewError ? <p className="inline-error" role="status">{previewError}</p> : preview === null ? <p className="inline-status" role="status">Loading preview…</p> : <>
              {previewHighlights?.kind === "plain" && <p className="inline-status" role="status">Search matches could not be verified, so this exact revision is shown without highlights.</p>}
              <pre className="resource-preview-content">{previewHighlights?.kind === "highlighted"
                ? previewHighlights.segments.map((segment, index) => segment.terms.length > 0
                  ? <mark key={`${segment.terms.join("-")}-${index}`} title={`Indexed term: ${segment.terms.join(", ")}`}>{segment.text}</mark>
                  : <span key={`plain-${index}`}>{segment.text}</span>)
                : preview}</pre>
            </>}
          </section>}
        </>
      )}
      {revisionResource && <ResourceRevisionEditor resource={revisionResource} activeWorkspaceId={selectedWorkspaceId} onClose={() => setRevisionResource(null)} onSelectResource={setRevisionResource} onSelectWorkspace={onSelectWorkspace} onNavigationSaveFailure={onDraftSaveFailure} onCommitted={(updated) => { onRevisionCommitted(updated); setRevisionResource((current) => current?.workspaceId === updated.workspaceId && current.resourceId === updated.resourceId ? updated : current); }} />}
      <section className="workspace-list-card" aria-labelledby="finished-artifacts-heading">
        <div className="section-heading"><div><h2 id="finished-artifacts-heading">Finished Artifacts</h2><p>Committed outputs from work in this Workspace, with version history and provenance.</p></div></div>
        {!selectedWorkspaceId
          ? <p className="inline-status" role="status">Select a Workspace to view its Artifacts.</p>
          : !operatorReady
            ? <p className="inline-status" role="status">Start the local Runtime to load committed Artifacts.</p>
            : <ArtifactLibrary api={artifactApi} workspaceId={selectedWorkspaceId} onOpenSource={onOpenArtifactSource} />}
      </section>
    </div>
  );
}

