import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

/**
 * A-26 empty-state truthfulness guard (docs/V1-UI-SCREEN-CONTRACT.md "Conversation screen").
 *
 * The no-selection / no-message Conversation canvas must render a truthful, LiteCowork-branded
 * empty state. It must NOT carry third-party "Antigravity" branding and must NOT advertise any
 * capability the current build cannot perform. This is the chat-side analogue of
 * workbench-truthfulness.test.mjs; it reads the component source so the removed placeholder
 * cannot silently return.
 */

const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const conversationsPagePath = join(packageRoot, "src", "conversations", "ConversationsPage.tsx");

const source = await readFile(conversationsPagePath, "utf8");

/**
 * Isolate the empty-state JSX block so the guard checks what actually renders, not the
 * explanatory comment that documents why the placeholder was removed (which legitimately
 * names "Antigravity" to say it must not appear).
 */
function emptyStateMarkup(src) {
  const anchor = src.indexOf("conversation-empty-state");
  assert.notEqual(anchor, -1, "the empty-state container className conversation-empty-state must exist");
  // Capture a generous window around the rendered block (the <div> body only), excluding comments.
  const windowText = src.slice(anchor, anchor + 1200);
  return windowText.replace(/\{\/\*[\s\S]*?\*\/\}/g, "");
}

const BANNED_BRANDING = ["Antigravity"];

// Capabilities this build cannot perform and must not advertise in an empty chat canvas.
const FABRICATED_CAPABILITY_CLAIMS = [
  "verifiable command",
  "run test suite",
  "inspect terminal output",
  "zero hallucination",
  "Execute verifiable",
];

test("the empty Conversation canvas carries no third-party Antigravity branding", () => {
  const markup = emptyStateMarkup(source);
  for (const banned of BANNED_BRANDING) {
    assert.equal(
      markup.includes(banned),
      false,
      `empty-state markup must not contain third-party branding: ${banned}`,
    );
  }
});

test("the empty Conversation canvas advertises no capability this build cannot perform", () => {
  const markup = emptyStateMarkup(source);
  for (const claim of FABRICATED_CAPABILITY_CLAIMS) {
    assert.equal(
      markup.toLowerCase().includes(claim.toLowerCase()),
      false,
      `empty-state markup must not advertise unsupported capability: ${claim}`,
    );
  }
});

test("the Send control is gated on a qualified binding, not merely a non-empty draft", () => {
  // A-24 disabled-until-qualified gate: the Send button's disabled expression must depend on the
  // composed eligibility predicate (which includes binding qualification), not only draft/operator.
  assert.match(
    source,
    /disabled=\{!sendEligible\}/,
    "Send button must be disabled unless sendEligible (includes a qualified binding) is true",
  );
  assert.match(
    source,
    /bindingEligibleForSend/,
    "send eligibility must incorporate a binding-qualification predicate",
  );
});
