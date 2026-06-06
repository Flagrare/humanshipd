# humanshipd browser extension (POC)

A thin Chrome (MV3) adapter that watches how you write in a web text field, then
asks the local humanshipd host to issue a **Human Authored** credential for it.
It holds no credential logic — capture happens in the page, signing happens in
the local host. The captured text is sent only to that local host (to compute a
hash); nothing is transmitted off your machine.

It targets ordinary web editors (a `<textarea>`, an `<input>`, or any
`contenteditable` element). Google Docs renders to a `<canvas>`, so its text
isn't readable from a content script — a Docs-specific capture path is future work.

## Try it

1. **Build the host:** `cargo build -p humanshipd-host`
2. **Load the extension:** open `chrome://extensions`, turn on *Developer mode*,
   click *Load unpacked*, and select this `extension/` folder. Copy the
   extension's ID that appears.
3. **Register the host** with that ID:
   `bash extension/host/install.sh <EXTENSION_ID>` then fully quit and reopen Chrome.
4. **Use it:** open any page with a text box (or `data:text/html,<textarea
   style=width:90%;height:300px>`), type a few sentences, then click the
   extension icon → *Issue Human Authored credential*. A `humanshipd-credential.c2pa`
   downloads.
5. **Verify it** against the text you wrote (save that text to `doc.txt`):
   `cargo run --example verify_credential -- humanshipd-credential.c2pa doc.txt`

Paste a chunk of text mid-session to see the AI-paste signal: the credential's
`ai_dump_flags` count rises and the claim flips to the warning.

## Tests

The capture logic (typed vs. pasted classification) has a Playwright regression
that injects the real `content.js` into a page and drives typing + a paste:

```bash
cd extension/tests && npm i && npx playwright install chromium && npm test
```

The extension → host → credential → AI-dump path is also covered by a Rust test
(`cargo test -p humanshipd-host`), so the capture↔host contract is guarded in CI
without a browser.

