// Regression for the Google Docs live-capture adapter (Decisions 1 + 2), codifying
// the logic so it can be re-checked without a live Google account: the isolated
// gdocs.js replays /save mutation ops into the document + event stream, flags a
// paste as a keystroke-less insertion, and the MAIN-world gdocs-inject.js pulls
// those ops out of a save POST body.
//
// Run: cd extension/tests && npm i && npx playwright install chromium && npm test
//
// The live end-to-end (real injection on docs.google.com + real paste) is validated
// by hand against a logged-in test account; this file covers the parse/replay/merge.

const { test, expect } = require("@playwright/test");
const fs = require("fs");
const path = require("path");

const read = (f) => fs.readFileSync(path.join(__dirname, "..", f), "utf8");
const GDOCS = read("gdocs.js");
const INJECT = read("gdocs-inject.js");

// Load gdocs.js (and optionally the MAIN-world inject) into a blank page with a
// chrome shim, exposing window.__session() and window.__postOp().
async function boot(page, { withInject = false } = {}) {
  await page.goto("data:text/html,<body></body>");
  await page.evaluate(
    ({ gdocs, inject, withInject }) => {
      let getSession = null;
      window.chrome = { runtime: { onMessage: { addListener: (fn) => (getSession = fn) } } };
      // Safe: `inject`/`gdocs` are our OWN extension scripts read from disk in this
      // test (not external/user input); eval loads the real scripts into the page,
      // mirroring extension/tests/capture.spec.js.
      if (withInject) {
        window.fetch = () => Promise.resolve(); // stub network; the inject wraps this
        // eslint-disable-next-line no-eval
        eval(inject);
      }
      // eslint-disable-next-line no-eval
      eval(gdocs);
      window.__session = () => {
        let s = null;
        if (getSession) getSession({ type: "getSession" }, null, (r) => (s = r));
        return s;
      };
      window.__postOp = (op, at) =>
        window.postMessage(
          { source: "humanshipd-gdocs", kind: "op", op, at: at || Date.now() },
          "*"
        );
    },
    { gdocs: GDOCS, inject: INJECT, withInject }
  );
}

// Let queued postMessage events flush before reading the session.
const flush = (page) => page.evaluate(() => new Promise((r) => setTimeout(r, 30)));

test("replays is / ds / mlti into the final text, all keystroke-backed", async ({ page }) => {
  await boot(page);
  await page.evaluate(() => {
    window.__postOp({ ty: "is", ibi: 1, s: "Hello " });
    window.__postOp({ ty: "mlti", mts: [{ ty: "is", ibi: 7, s: "word" }] });
    window.__postOp({ ty: "ds", si: 10, ei: 10 }); // delete the 'd' of "word"
    window.__postOp({ ty: "is", ibi: 10, s: "ld" }); // -> "Hello world"
  });
  await flush(page);

  const s = await page.evaluate(() => window.__session());
  expect(s.surface_kind).toBe("gdocs");
  expect(s.final_text).toBe("Hello world");
  // No insertion arrived without keystrokes — nothing should be flagged as a paste.
  const unkeyed = s.events.filter((e) => e.inserted_chars > 0 && e.keystrokes === 0);
  expect(unkeyed).toHaveLength(0);
});

test("flags a paste as the only keystroke-less insertion", async ({ page }) => {
  await boot(page);
  await page.evaluate(() => {
    window.__postOp({ ty: "is", ibi: 1, s: "typed " }); // typed
    const dt = new DataTransfer();
    dt.setData("text/plain", "PASTED BLOCK");
    document.dispatchEvent(new ClipboardEvent("paste", { clipboardData: dt, bubbles: true }));
    window.__postOp({ ty: "is", ibi: 7, s: "PASTED BLOCK" }); // the pasted insert
  });
  await flush(page);

  const s = await page.evaluate(() => window.__session());
  expect(s.final_text).toBe("typed PASTED BLOCK");
  const unkeyed = s.events.filter((e) => e.inserted_chars > 0 && e.keystrokes === 0);
  expect(unkeyed).toHaveLength(1);
  expect(unkeyed[0].inserted_chars).toBe("PASTED BLOCK".length);
});

test("the inject extracts ops from a /save body and feeds the session", async ({ page }) => {
  await boot(page, { withInject: true });
  await page.evaluate(async () => {
    const bundles = [
      { commands: [{ ty: "is", ibi: 1, s: "From save " }, { ty: "is", ibi: 11, s: "body" }] },
    ];
    const body = "rev=5&bundles=" + encodeURIComponent(JSON.stringify(bundles));
    await window.fetch("https://docs.google.com/document/d/x/save?id=1", { method: "POST", body });
  });
  await flush(page);

  const s = await page.evaluate(() => window.__session());
  expect(s.final_text).toBe("From save body");
});

// Drop a captured real save into fixtures/gdocs-save.json as { body, expected_text }
// to turn this into a real-data regression (and a parity check vs. the Rust parser).
const SAMPLE = path.join(__dirname, "fixtures", "gdocs-save.json");
test("matches the expected text on a real /save sample", async ({ page }) => {
  test.skip(!fs.existsSync(SAMPLE), "no fixtures/gdocs-save.json — see comment");
  const { body, expected_text } = JSON.parse(fs.readFileSync(SAMPLE, "utf8"));
  await boot(page, { withInject: true });
  await page.evaluate(async (b) => {
    await window.fetch("https://docs.google.com/save", { method: "POST", body: b });
  }, body);
  await flush(page);

  const s = await page.evaluate(() => window.__session());
  expect(s.final_text).toBe(expected_text);
});
