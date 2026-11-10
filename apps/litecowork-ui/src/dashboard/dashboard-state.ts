// Pure Workspace Dashboard state (011-T22). A widget shows a value only when a served owner
// projection returned it; otherwise it is UNAVAILABLE with a reason, never empty or invented.

export const WIDGET_IDS = ["tasks", "automations", "coworkers", "artifacts"] as const;
export type WidgetId = (typeof WIDGET_IDS)[number];

export const WIDGET_TITLES: Record<WidgetId, string> = {
  tasks: "Task status",
  automations: "Automation status",
  coworkers: "Coworker activity",
  artifacts: "Recent artifacts",
};

export type WidgetLine = { label: string; value: number };

export type WidgetState =
  | { status: "LOADING" }
  | { status: "UNAVAILABLE"; reason: string }
  | { status: "READY"; lines: WidgetLine[]; note: string | null };

export type KeyValueStorage = { getItem(key: string): string | null; setItem(key: string, value: string): void };

export const DASHBOARD_PINS_KEY = "litecowork.dashboard.pins.v1";
const MAX_WORKSPACES = 100;

function readAll(storage: KeyValueStorage): Record<string, WidgetId[]> {
  try {
    const raw = storage.getItem(DASHBOARD_PINS_KEY);
    if (!raw) return {};
    const parsed: unknown = JSON.parse(raw);
    if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) return {};
    const result: Record<string, WidgetId[]> = {};
    for (const [workspaceId, value] of Object.entries(parsed as Record<string, unknown>)) {
      if (!Array.isArray(value)) continue;
      const pins = WIDGET_IDS.filter((id) => value.includes(id));
      result[workspaceId] = pins;
    }
    return result;
  } catch {
    return {};
  }
}

/** Pins are stored per Workspace. A Workspace with no saved state shows every widget. */
export function loadPins(storage: KeyValueStorage, workspaceId: string): WidgetId[] {
  const saved = readAll(storage)[workspaceId];
  return saved ? [...saved] : [...WIDGET_IDS];
}

export function savePins(storage: KeyValueStorage, workspaceId: string, pins: readonly WidgetId[]): void {
  const all = readAll(storage);
  delete all[workspaceId];
  const entries = Object.entries(all).slice(-(MAX_WORKSPACES - 1));
  const next = Object.fromEntries([...entries, [workspaceId, WIDGET_IDS.filter((id) => pins.includes(id))]]);
  try {
    storage.setItem(DASHBOARD_PINS_KEY, JSON.stringify(next));
  } catch {
    // Persistence is best effort; the in-memory pin state still applies for this session.
  }
}

export function togglePin(pins: readonly WidgetId[], id: WidgetId): WidgetId[] {
  return pins.includes(id) ? pins.filter((pin) => pin !== id) : WIDGET_IDS.filter((widget) => widget === id || pins.includes(widget));
}

export function unavailable(reason: string): WidgetState {
  return { status: "UNAVAILABLE", reason };
}

/** Counts items by a status-like key. `truncated` means more pages exist, so counts are lower bounds. */
export function summarizeByStatus(
  items: readonly { status: string }[],
  labels: Readonly<Record<string, string>>,
  truncated: boolean,
): WidgetState {
  const counts = new Map<string, number>();
  for (const item of items) counts.set(item.status, (counts.get(item.status) ?? 0) + 1);
  const lines = [...counts.entries()]
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([status, value]) => ({ label: labels[status] ?? status, value }));
  return {
    status: "READY",
    lines,
    note: truncated ? "Showing the first page only; there are more items than counted here." : null,
  };
}

export function summarizeTotal(label: string, total: number, truncated: boolean): WidgetState {
  return {
    status: "READY",
    lines: [{ label, value: total }],
    note: truncated ? "Showing the first page only; there are more items than counted here." : null,
  };
}
