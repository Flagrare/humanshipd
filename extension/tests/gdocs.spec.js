// Regression for the Google Docs live-capture adapter: gdocs.js records normalized
// ops into a capture log and flags pastes as keystroke-less insertions. The
// MAIN-world gdocs-inject.js extracts ops from /save POST bodies and feeds them
// into the session.
//
// Run: cd extension/tests && npm i && npx playwright install chromium && npm test
//
// The live end-to-end (real injection on docs.google.com + real paste) is validated
// by hand against a logged-in test account; this file covers the parse/capture/merge.

const { test, expect } = require("@playwright/test");
const fs = require("fs");
const path = require("path");

const read = (f) => fs.readFileSync(path.join(__dirname, "..", f), "utf8");
// Patch location.pathname to a fixed doc path so docId resolves without needing to
// redefine window.location (Chromium blocks that even in Playwright page contexts).
const GDOCS = read("gdocs.js").replace("location.pathname", '"/document/d/DOC/edit"');
const INJECT = read("gdocs-inject.js");

// Load gdocs.js (and optionally the MAIN-world inject) into a blank page with a
// chrome shim, exposing window.__session() and window.__postOp().
async function boot(page, { withInject = false } = {}) {
  await page.goto("data:text/html,<body></body>");
  await page.evaluate(
    ({ gdocs, inject, withInject }) => {
      window.__store = window.__store || {};
      let getSession = null;
      window.chrome = {
        runtime: { onMessage: { addListener: (fn) => (getSession = fn) } },
        storage: { local: {
          set: (obj) => Object.assign(window.__store, obj),
          get: (_k, cb) => cb({ ...window.__store }),
        } },
      };
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

test("flags a paste as the only keystroke-less op", async ({ page }) => {
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
  const ops = s.log.sessions.at(-1).ops;
  const pasted = ops.filter((o) => o.pasted === true);
  expect(pasted).toHaveLength(1);
  expect(Array.from(pasted[0].text).length).toBe("PASTED BLOCK".length);
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
  const ops = s.log.sessions.at(-1).ops;
  expect(ops[0].text).toBe("From save ");
  expect(ops[1].text).toBe("body");
});
