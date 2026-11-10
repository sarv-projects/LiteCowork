import assert from "node:assert/strict";
import { after, test } from "node:test";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import react from "@vitejs/plugin-react";
import { createServer } from "vite";

const vite = await createServer({
  configFile: false,
  root: fileURLToPath(new URL("..", import.meta.url)),
  plugins: [react()],
  server: { middlewareMode: true, hmr: false },
  optimizeDeps: { noDiscovery: true, include: [] },
  appType: "custom",
});
const { UpdatePanel } = await vite.ssrLoadModule("/src/updates/UpdatePanel.tsx");
after(async () => vite.close());

const api = {
  async checkForUpdate() { throw new Error("unused"); },
  async installUpdate() { throw new Error("unused"); },
};

function render(platform) {
  return renderToStaticMarkup(createElement(UpdatePanel, { api, platform }));
}

test("signed Windows build starts idle with install disabled and a visible reason", () => {
  const html = render({ os: "windows", signed: true });
  assert.match(html, /data-update-state="IDLE"/);
  assert.match(html, /role="status"/);
  assert.match(html, /aria-live="polite"/);
  assert.match(html, /tabindex="-1"/i);
  assert.match(html, /<button[^>]*disabled=""[^>]*aria-describedby="update-install-reason"/);
  assert.match(html, /id="update-install-reason"[^>]*>Install is unavailable/);
  assert.doesNotMatch(html, /up to date/i);
});

test("unsigned build disables both commands and says why", () => {
  const html = render({ os: "windows", signed: false });
  assert.match(html, /unsigned development build/);
  assert.equal((html.match(/disabled=""/g) ?? []).length, 2);
});

test("Linux row tells the user to install the new package", () => {
  assert.match(render({ os: "linux", signed: true }), /Update by installing the new package/);
});

test("panel adds no animation and keeps a visible focus style", async () => {
  const css = await readFile(new URL("../src/styles.css", import.meta.url), "utf8");
  assert.match(css, /\.update-panel-message:focus-visible/);
  const block = css.slice(css.indexOf(".update-panel {"));
  assert.doesNotMatch(block.slice(0, block.indexOf("@media") === -1 ? undefined : block.indexOf("@media")), /animation|transition/);
});
