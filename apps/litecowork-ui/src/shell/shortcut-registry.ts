/**
 * One registry for app-wide shortcuts (011-T11). The sidebar label and the key matcher both
 * derive from these definitions, and App must provide a handler for every id (the handler
 * map is typed `Record<ShortcutId, ...>`), so a label cannot exist without a handler.
 *
 * Ctrl + W is deliberately not registered: WebViews and operating systems reserve it for
 * closing the window or tab, so the Workbench toggle stays a button until an alternative
 * is documented.
 */
export const SHORTCUTS = [
  { id: "new-chat", key: "n", label: "New Chat" },
  { id: "open-library", key: "k", label: "Library and search" },
] as const;

export type ShortcutId = (typeof SHORTCUTS)[number]["id"];

export type ShortcutEventLike = {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  defaultPrevented?: boolean;
};

export function shortcutKeys(id: ShortcutId): string {
  const definition = SHORTCUTS.find(item => item.id === id);
  return definition ? `Ctrl ${definition.key.toUpperCase()}` : "";
}

export function matchShortcut(event: ShortcutEventLike): ShortcutId | null {
  if (event.defaultPrevented || event.altKey || event.shiftKey) return null;
  if (!(event.ctrlKey || event.metaKey)) return null;
  const key = event.key.toLowerCase();
  return SHORTCUTS.find(item => item.key === key)?.id ?? null;
}
