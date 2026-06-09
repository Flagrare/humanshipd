// Regression for cross-session continuity: gdocs.js records normalized ops, persists
// them to a (shimmed) chrome.storage, and on a reload resumes — so getSession returns
// a log accumulating BOTH the prior session and the new one.

const { test, expect } = require("@playwright/test");
const fs = require("fs");
const path = require("path");

const GDOCS = fs.readFileSync(path.join(__dirname, "..", "gdocs.js"), "utf8");

// Load gdocs.js with a chrome shim backed by a JS object that survives a "reload".
// The script text is patched to substitute a fixed doc pathname so docId resolves
// without needing to redefine window.location (which Chromium blocks).
const GDOCS_PATCHED = GDOCS.replace(
  "location.pathname",
  '"/document/d/DOC123/edit"'
);

async function boot(page, store) {
  await page.goto("data:text/html,<body></body>");
  await page.evaluate(({ gdocs, store }) => {
    window.__store = store;
    let getSession = null;
    window.chrome = {
      runtime: { onMessage: { addListener: (fn) => (getSession = fn) } },
      storage: {
        local: {
          // Safe: gdocs is our own extension script read from disk (not user input).
          set: (obj) => Object.assign(window.__store, obj),
          get: (_keys, cb) => cb({ ...window.__store }),
        },
      },
    };
    // eslint-disable-next-line no-eval
    eval(gdocs);
    window.__post = (op, at) => window.postMessage({ source: "humanshipd-gdocs", kind: "op", op, at: at || Date.now() }, "*");
    window.__session = () => { let s = null; if (getSession) getSession({ type: "getSession" }, null, (r) => (s = r)); return s; };
  }, { gdocs: GDOCS_PATCHED, store });
}

const flush = (page) => page.evaluate(() => new Promise((r) => setTimeout(r, 30)));

test("resumes a prior session and accumulates the new one", async ({ page }) => {
  const store = {};
  // Session 1: type "Hello".
  await boot(page, store);
  await page.evaluate(() => window.__post({ ty: "is", ibi: 1, s: "Hello" }));
  await flush(page);
  await page.evaluate(() => new Promise((r) => setTimeout(r, 1600))); // let debounced save fire
  const after1 = await page.evaluate(() => JSON.parse(JSON.stringify(window.__store)));
  expect(Object.keys(after1).some((k) => k.includes("DOC123:s0"))).toBe(true);

  // "Reload": new page, SAME store object → prior session loads, new session appends.
  await boot(page, after1);
  await page.evaluate(() => window.__post({ ty: "is", ibi: 6, s: " world" }));
  await flush(page);
  const out = await page.evaluate(() => window.__session());
  expect(out.log.sessions.length).toBe(2);
  expect(out.log.sessions[0].ops[0].text).toBe("Hello");
  expect(out.log.sessions[1].ops[0].text).toBe(" world");
});
