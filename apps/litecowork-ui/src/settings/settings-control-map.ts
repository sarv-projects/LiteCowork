/**
 * One entry per interactive control rendered directly by SettingsView (011-T06, B-29/B-33).
 * Classes follow docs/V1-SETTINGS-CONTRACT-LEDGER.md "Pinned V1 interactive control set".
 * A control may only be interactive if it names a served operation or an applied local effect.
 */
export type SettingsControlClass = "LocalOnly" | "Native" | "Target" | "OperatorServed";

export type SettingsControlEntry = {
  class: SettingsControlClass;
  /** Served operationId, desktop bridge command, or local effect. */
  effect: string;
};

export const SETTINGS_CONTROL_MAP: Readonly<Record<string, SettingsControlEntry>> = {
  "nav-general": { class: "LocalOnly", effect: "category navigation" },
  "nav-workspaces": { class: "LocalOnly", effect: "category navigation" },
  "nav-agents": { class: "LocalOnly", effect: "category navigation" },
  "nav-notifications": { class: "LocalOnly", effect: "category navigation" },
  "notifications-open": { class: "LocalOnly", effect: "navigate to the Notifications page (Feature 010 owns quiet hours)" },
  "nav-runtime": { class: "LocalOnly", effect: "category navigation" },
  "pref-theme": { class: "LocalOnly", effect: "LocalPreferenceStore theme applied at document root" },
  "pref-density": { class: "LocalOnly", effect: "LocalPreferenceStore density applied at document root" },
  "pref-text-scale": { class: "LocalOnly", effect: "LocalPreferenceStore text scale applied at document root" },
  "pref-reduced-motion": { class: "LocalOnly", effect: "LocalPreferenceStore reduced motion applied at document root" },
  "workspaces-refresh": { class: "OperatorServed", effect: "listWorkspaces" },
  "workspaces-select": { class: "LocalOnly", effect: "selected Workspace in desktop state" },
  "workspaces-create-name": { class: "OperatorServed", effect: "createWorkspace draft" },
  "workspaces-create-submit": { class: "OperatorServed", effect: "createWorkspace" },
  "workspaces-instructions-text": { class: "OperatorServed", effect: "Workspace instructions draft" },
  "workspaces-instructions-save": { class: "OperatorServed", effect: "createWorkspaceInstructionRevision" },
  "workspaces-archive-cancel": { class: "LocalOnly", effect: "dismiss archive confirmation" },
  "workspaces-archive-confirm": { class: "OperatorServed", effect: "archive_workspace" },
  "workspaces-archive-begin": { class: "LocalOnly", effect: "open two-step archive confirmation" },
  "workspaces-rename-name": { class: "OperatorServed", effect: "renameWorkspace draft" },
  "workspaces-rename-submit": { class: "OperatorServed", effect: "renameWorkspace" },
  "runtime-start": { class: "Native", effect: "start_local_runtime desktop bridge" },
  "runtime-refresh": { class: "OperatorServed", effect: "getLocalRuntimeLifecycle" },
};
