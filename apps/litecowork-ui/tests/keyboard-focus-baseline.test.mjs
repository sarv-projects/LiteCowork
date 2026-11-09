import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { SHORTCUTS, matchShortcut, shortcutKeys } from "../src/shell/shortcut-registry.ts";

const app = await readFile(new URL("../src/App.tsx", import.meta.url), "utf8");
const event = (key, overrides = {}) => ({ key, ctrlKey: true, metaKey: false, altKey: false, shiftKey: false, ...overrides });

test("every registered shortcut has a handler in App (AC-018)", () => {
  const block = app.match(/const shortcutHandlers: Record<ShortcutId, \(\) => void> = \{([\s\S]*?)\n  \};/);
  assert.ok(block, "shortcutHandlers map present");
  for (const { id } of SHORTCUTS) {
    assert.match(block[1], new RegExp(`"${id}":`), `no handler for ${id}`);
  }
});

test("shortcut labels shown in the UI come from the registry, never literals", () => {
  assert.doesNotMatch(app, />\s*Ctrl [A-Z]\s*</);
  assert.match(app, /shortcutKeys\("new-chat"\)/);
  assert.equal(shortcutKeys("new-chat"), "Ctrl N");
  assert.equal(shortcutKeys("open-library"), "Ctrl K");
});

test("matcher accepts Ctrl or Meta and ignores modified, plain or unknown keys", () => {
  assert.equal(matchShortcut(event("n")), "new-chat");
  assert.equal(matchShortcut(event("K")), "open-library");
  assert.equal(matchShortcut(event("n", { ctrlKey: false, metaKey: true })), "new-chat");
  assert.equal(matchShortcut(event("n", { ctrlKey: false })), null);
  assert.equal(matchShortcut(event("n", { shiftKey: true })), null);
  assert.equal(matchShortcut(event("n", { altKey: true })), null);
  assert.equal(matchShortcut(event("w")), null, "Ctrl W is reserved by the WebView and stays unregistered");
  assert.equal(matchShortcut(event("n", { defaultPrevented: true })), null);
});
