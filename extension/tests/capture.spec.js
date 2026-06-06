// Regression for the content-script capture logic, codifying the smoke-test
// trajectory: typed input must produce keystroke-backed events, and a paste must
// be recorded as a single keystroke-less insertion (the AI-dump signal).
//
// Run: cd extension/tests && npm i && npx playwright install chromium && npm test
//
// (This same sequence was validated via the Playwright MCP during development;
// this file lets anyone re-run it without driving the browser by hand.)

const { test, expect } = require("@playwright/test");
const fs = require("fs");
const path = require("path");

test("classifies typed input vs a paste", async ({ page }) => {
  await page.goto(
    'data:text/html,<textarea id="ed" style="width:90%;height:200px"></textarea>'
  );

  // Inject the REAL content.js with a chrome shim that captures its listener.
  const source = fs.readFileSync(path.join(__dirname, "..", "content.js"), "utf8");
  await page.evaluate((src) => {
    let captured = null;
    window.chrome = { runtime: { onMessage: { addListener: (fn) => (captured = fn) } } };
    // Safe: `src` is our own content.js read from disk in this test, evaluated to
    // load the real content script into the page (not external/user input).
    // eslint-disable-next-line no-eval
    eval(src);
    window.__getSession = () => {
      let s = null;
      captured({ type: "getSession" }, null, (r) => (s = r));
      return s;
    };
  }, source);

  // Type real keystrokes.
  await page.locator("#ed").pressSequentially("the quick brown fox");

  // Simulate a paste: a 'paste' event, then a value jump + 'input'.
  await page.evaluate(() => {
    const ed = document.querySelector("#ed");
    ed.focus();
    ed.dispatchEvent(new ClipboardEvent("paste", { bubbles: true }));
    ed.value += " PASTED-AI-BLOCK-OF-FORTY-PLUS-CHARACTERS-HERE";
    ed.dispatchEvent(new InputEvent("input", { bubbles: true }));
  });

  const session = await page.evaluate(() => window.__getSession());
  const typed = session.events.filter((e) => e.keystrokes > 0);
  const unkeyed = session.events.filter((e) => e.inserted_chars > 0 && e.keystrokes === 0);

  expect(typed.length).toBeGreaterThan(0); // typing is keystroke-backed
  expect(unkeyed).toHaveLength(1); // the paste is the only keystroke-less insertion
  expect(unkeyed[0].inserted_chars).toBeGreaterThanOrEqual(20); // large enough to flag
});
