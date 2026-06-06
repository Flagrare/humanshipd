# Research: Capturing the Google Docs Writing Process in the Browser

- **Slug:** `2026-06-06-google-docs-writing-capture`
- **Date:** 2026-06-06
- **Status:** complete — both mechanisms confirmed live on a real authenticated doc (2026-06-06)
- **Triggered by:** Decision to make Google Docs the "smoking gun" — prove browser capture works on the hardest, most-wanted surface before refactoring.
- **Informed:** Capture-fidelity design ([`2026-06-06-content-binding-and-capture-fidelity-design.md`](../superpowers/specs/2026-06-06-content-binding-and-capture-fidelity-design.md)); the forthcoming Google Docs capture-adapter spike.

## Question

How is a Google Doc's *writing process* (the stream of insert/delete edits with positions and timestamps) actually captured in the browser — given Docs renders to `<canvas>` — and what is the most robust mechanism for a content-script extension to feed our `EditEvent` pipeline?

## Sources

### [How I reverse-engineered Google Docs — James Somers](https://features.jsomers.net/how-i-reverse-engineered-google-docs/)
- **Type:** engineering blog — **primary** · **Accessed:** 2026-06-06 · **Relevance:** high
- **What this contributed:** The canonical recipe. The full keystroke-level log is at the internal `GET https://docs.google.com/document/d/<ID>/revisions/load?id=<ID>&start=<N>&end=<M>` (account-scoped `/u/<i>/` when multi-signed-in). Response is prefixed with the anti-hijack guard `)]}'` then JSON with a `changelog` array (and `chunkedSnapshot` baseline). Ops: `ty:"is"` insert (`ibi` index, `s` string), `ty:"ds"` delete (`si`/`ei`), `ty:"mlti"` multi-bundle; per-entry timestamp + session/user. Latest revision found by binary search (out-of-range → HTTP 500). Auth is **session cookies only** for the GET. Also the key live insight: Docs sends mutations to the server on a `save`/`bind` call "every time I typed," and the server "gets no more data than what is sent via these save calls" — so intercepting those is sufficient.

### Live-capture mechanism (MV3 main-world fetch/XHR patch)
- **Sources:** [Chrome `content_scripts` `world: "MAIN"` (Chrome 111+)](https://developer.chrome.com/docs/extensions/reference/manifest/content-scripts), [Inject a Global in MV3 — David Walsh](https://davidwalsh.name/inject-global-mv3), [Google Docs canvas rendering (Lobsters)](https://lobste.rs/s/uqb3kj/google_docs_will_now_use_canvas_based) — **primary/secondary** · **Accessed:** 2026-06-06 · **Relevance:** high
- **What this contributed:** Since the isolated content-script world can't see the page's `fetch`/`XHR`, intercepting `/save` requires a **MAIN-world** content script at `run_at: "document_start"` (manifest `world:"MAIN"` or `chrome.scripting.registerContentScripts`), patching `fetch`/`XMLHttpRequest` before Docs grabs its reference, then bridging to the isolated world via `postMessage`. DOM `MutationObserver` is dead post-canvas (2021); the "screen reader support" accessibility region exposes a partial, caret-window text (not positioned ops) — fallback/cross-check only. Persist each event immediately (the MV3 service worker dies after ~30s idle).

### Official Google APIs — the gap
- **Sources:** [Drive `revisions` reference](https://developers.google.com/workspace/drive/api/reference/rest/v3/revisions/list), [Docs API `documents.get`](https://developers.google.com/workspace/docs/api/reference/rest/v1/documents/get), [Apps Script installable triggers](https://developers.google.com/apps-script/guides/triggers/installable) — **primary** · **Accessed:** 2026-06-06 · **Relevance:** high
- **What this contributed:** No official API exposes edit-level process data. Drive `revisions` are coarse merged snapshots ("list may be incomplete… older revisions omitted"), metadata only (no ops/diffs). The Docs API returns current state, no history. Docs has no `onEdit` trigger (Sheets-only). Drive scopes are *restricted* → annual CASA audit. Conclusion: the undocumented internal endpoints are the only fine-grained path.

### Competing tools' Docs method + open-source references
- **Sources:** [Originality.ai "watch a writer write"](https://originality.ai/blog/chatgpt-detection-chrome-extension-watch-writer), [GPTZero Docs](https://gptzero.me/news/announcing-gptzero-docs-the-future-of-transparent-writing/), [Brisk Inspect Writing](https://www.briskteaching.com/inspect-writing), [Grammarly Authorship](https://support.grammarly.com/hc/en-us/articles/29548735595405-About-Authorship), [harvard-vpal/gdocrevisions (MIT)](https://github.com/harvard-vpal/gdocrevisions), [kumofx/kumodocs](https://github.com/kumofx/kumodocs), [ArgLab/writing_observer (AGPL)](https://github.com/ArgLab/writing_observer) — **primary/secondary** · **Accessed:** 2026-06-06 · **Relevance:** high
- **What this contributed:** Draftback, Revision History, Brisk, GPTZero, and Originality.ai all use the **`revisions/load` after-the-fact** path (Originality confirms "using Google Docs revision history"). **Grammarly is the exception** — a *live* content-script capturing keystrokes + clipboard (with paste-source labeling), on-device AES-256-GCM, not dependent on the internal endpoint. Permission pattern: host permission for `docs.google.com/*` + a content script on that origin to issue `revisions/load` as a same-origin request; no `webRequest` needed. **`gdocrevisions` (MIT)** is the cleanest parser reference; `writing_observer` is **AGPL** (ideas only, don't copy).

## Live confirmation (2026-06-06, real authenticated doc)

Captured directly from a logged-in Google Doc, so these are the *current* formats, not folklore:

**Live `/save` POST** — `POST /document/u/0/d/<ID>/save?id=<ID>&sid=<sid>&token=<XSRF>&...&tab=t.0`, body (form-urlencoded):

```
rev=1&bundles=[{"commands":[{"ty":"is","ibi":1,"s":"Th"}],"sid":"<sid>","reqId":0}]
```

So a save carries `rev` + a `bundles` array; each bundle has `commands` (the ops), `sid`, `reqId`. The op confirmed: `{"ty":"is","ibi":<1-based index>,"s":"<text>"}` (insert). Captured live as typing happened. After the first save, subsequent edits route over the persistent `bind` RPC channel — so a live capturer should patch both `/save` and the `bind` send path.

**After-the-fact `/revisions/load`** — `GET /document/u/0/d/<ID>/revisions/load?id=<ID>&start=1&end=<N>&token=<XSRF>&tab=t.0`. Required params that the older writeups omit: the **`/u/0/` account prefix**, the **`token`** (XSRF, same one the save URL carries), and **`tab=t.0`** (the new tabbed-docs param) — without them it returns 400. Response begins with `)]}'` then JSON: `{"chunkedSnapshot":[[ ... style ops ty:"as" ... ]], "changelog":[[{"ty":"m...` — `chunkedSnapshot` is the baseline state at `start`, `changelog` is the op stream to replay.

**Implication:** both the live and historical paths are reachable from a content script on `docs.google.com` (the live one needs MAIN-world fetch/XHR patching; the historical one is a same-origin GET with the page's token). The recommended build is live `/save`+`bind` interception, with `/revisions/load` as the "credential for an already-written doc" import.

## Synthesis

Google Docs capture in the browser is settled-feasible — you don't read the canvas, you read the **mutation stream**. Two mechanisms, and they're complementary:

1. **Live (recommended primary):** a MAIN-world `document_start` content script patches `fetch`/`XHR` and parses the `/save` mutation bundles as the user types → `{op, position, length, timestamp}` into our `EditEvent` pipeline. Most robust (Google's own persistence path), gives true offsets natively, and works the instant typing starts. Grammarly validates the live-capture posture.
2. **After-the-fact (secondary/enrichment):** fetch `/revisions/load` (same-origin from a `docs.google.com` content script), strip `)]}'`, replay `changelog` ops — for issuing a credential over an already-written doc. `gdocrevisions` (MIT) is the parser reference.

Official APIs are too coarse (snapshots, no ops, no live Docs trigger; restricted-scope/CASA burden), so the internal endpoints are the only fine-grained option — the maintenance cost (undocumented, drift-prone field names) is one every tool in this space pays. **Open item:** confirm the *current* `/save` request body and `/revisions/load` response field names against a real authenticated doc before writing the parser (planned via a live capture).

## Downstream uses

- Capture-fidelity spec §4 (capture adapters) — Google Docs adapter via live `/save` interception, `/revisions/load` import as secondary.
- Reframes the architecture's primary-capture-surface question: browser capture *is* viable for Docs via the right per-surface mechanism (not generic DOM).
