import { SHORTCUTS, shortcutKeys } from "./shortcut-registry.ts";

/** Generated from the shortcut registry, so a listed shortcut always has a handler (011-T11). */
export function ShortcutList() {
  return (
    <section className="settings-card" aria-labelledby="settings-shortcuts-title">
      <h2 id="settings-shortcuts-title" className="settings-card-title">Keyboard shortcuts</h2>
      <dl className="settings-details-grid">
        {SHORTCUTS.map((shortcut) => (
          <div key={shortcut.id}>
            <dt>{shortcut.label}</dt>
            <dd>{shortcutKeys(shortcut.id)}</dd>
          </div>
        ))}
      </dl>
      <p className="settings-card-subtitle">Ctrl W is reserved by the operating system and is not a LiteCowork shortcut.</p>
    </section>
  );
}
