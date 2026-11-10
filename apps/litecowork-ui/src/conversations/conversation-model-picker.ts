import type { AgentControlDescriptor, ConversationAgentSurface } from "./conversation-agent-surface.ts";

type SessionOption = NonNullable<AgentControlDescriptor["session_options"]>["options"][number];

export type ModelPickerLoadState = "LOADING" | "ERROR" | "DENIED" | "OFFLINE" | "READY";

export type ModelPickerItem = {
  providerModelId: string;
  label: string;
  selected: boolean;
  /** True only when the descriptor schema declares this value as its default. */
  isDefault: boolean;
};

export type ModelPickerView = {
  state:
    | "LOADING" | "ERROR" | "DENIED" | "OFFLINE" | "STALE" | "BLOCKED"
    | "EMPTY" | "READY";
  /** Plain-text status; never color-only. */
  message: string;
  refreshAvailable: boolean;
  optionId: string | null;
  items: ModelPickerItem[];
  /** Effort row is present only when the descriptor declares an effort option. */
  effort: { optionId: string; values: string[]; selected: string | null } | null;
  /** The listbox announcement for screen readers. */
  announcement: string;
};

export type ModelSelection = {
  optionId: string;
  providerModelId: string;
  descriptorRevision: string;
};

const NONE: Pick<ModelPickerView, "optionId" | "items" | "effort" | "announcement"> = {
  optionId: null,
  items: [],
  effort: null,
  announcement: "",
};

function enumValues(option: SessionOption): string[] {
  const schema = option.value_schema as { enum?: unknown } | null;
  const values = schema && Array.isArray(schema.enum) ? schema.enum : [];
  return values.filter((value): value is string => typeof value === "string" && value.length > 0);
}

function defaultValue(option: SessionOption): string | null {
  const schema = option.value_schema as { default?: unknown } | null;
  return schema && typeof schema.default === "string" ? schema.default : null;
}

/**
 * Projects the daemon descriptor into the composer model popover. TypeScript only
 * renders what the daemon declared: it recomputes no freshness and invents no model.
 * Options from one descriptor never mix with another provider's.
 */
export function deriveModelPicker(
  surface: ConversationAgentSurface,
  descriptor: AgentControlDescriptor | null,
  selected: { model: string | null; effort: string | null },
  load: ModelPickerLoadState = "READY",
): ModelPickerView {
  const states: Partial<Record<ModelPickerLoadState, [ModelPickerView["state"], string]>> = {
    LOADING: ["LOADING", "Loading this agent's model options."],
    ERROR: ["ERROR", "This agent's model options could not be read."],
    DENIED: ["DENIED", "You do not have access to this agent's model options."],
    OFFLINE: ["OFFLINE", "The local service is offline, so model options are unavailable."],
  };
  const loadMessage = states[load];
  if (loadMessage) {
    return { state: loadMessage[0], message: loadMessage[1], refreshAvailable: load !== "LOADING", ...NONE };
  }
  if (surface.status === "NEEDS_REFRESH" || !descriptor) {
    return {
      state: "STALE",
      message: "Model options are out of date. Refresh this agent to choose a model.",
      refreshAvailable: true,
      ...NONE,
    };
  }
  if (!surface.controls) {
    return { state: "BLOCKED", message: surface.blockers[0] ?? "Model options are unavailable.", refreshAvailable: false, ...NONE };
  }
  const options = surface.controls.options;
  const model = options.find(option => option.semantic_hint === "MODEL" && enumValues(option).length > 0);
  if (!model) {
    return { state: "EMPTY", message: "This agent declares no model options.", refreshAvailable: true, ...NONE };
  }
  const modelDefault = defaultValue(model);
  const items = enumValues(model).map(value => ({
    providerModelId: value,
    label: value,
    selected: value === selected.model,
    isDefault: value === modelDefault,
  }));
  const effortOption = options.find(option => option.semantic_hint === "EFFORT" && enumValues(option).length > 0);
  const chosen = items.find(item => item.selected);
  return {
    state: "READY",
    message: chosen ? `Model ${chosen.label} selected.` : "Choose a model.",
    refreshAvailable: true,
    optionId: model.option_id,
    items,
    effort: effortOption
      ? {
        optionId: effortOption.option_id,
        values: enumValues(effortOption),
        selected: enumValues(effortOption).includes(selected.effort ?? "") ? selected.effort : null,
      }
      : null,
    announcement: chosen ? `${chosen.label}, selected` : `${items.length} models available`,
  };
}

/** Builds the admission payload, refusing a model the current descriptor does not declare. */
export function buildModelSelection(
  view: ModelPickerView,
  descriptor: AgentControlDescriptor,
  providerModelId: string,
): ModelSelection | null {
  if (view.state !== "READY" || !view.optionId) return null;
  if (!view.items.some(item => item.providerModelId === providerModelId)) return null;
  return { optionId: view.optionId, providerModelId, descriptorRevision: descriptor.descriptor_digest };
}

/** The "/" affordance lists only declared commands and inserts nothing otherwise. */
export function deriveCommandList(surface: ConversationAgentSurface): {
  commands: { commandId: string; token: string; label: string }[];
  emptyMessage: string | null;
} {
  const declared = surface.controls?.commands ?? [];
  if (!declared.length) return { commands: [], emptyMessage: "No commands declared by this agent" };
  return {
    commands: declared.map(command => ({ commandId: command.command_id, token: command.token, label: command.display_name })),
    emptyMessage: null,
  };
}
