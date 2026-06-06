# humanshipd browser extension (POC)

A thin Chrome (MV3) adapter that watches how you write in a web text field, then
asks the local humanshipd host to issue a **Human Authored** credential for it.
It holds no credential logic — capture happens in the page, signing happens in
the local host. The captured text is sent only to that local host (to compute a
hash); nothing is transmitted off your machine.

It captures text typed **into a web page**, but not every web editor exposes its
text the same way:

- **Works:** real `<textarea>` / `<input>` fields, and rich `contenteditable`
  editors (tiptap, ProseMirror, Quill…), including ones inside `about:blank` /
  `designMode` iframes.
- **Refused, on purpose:** *code* editors (Ace, CodeMirror, Monaco) type into a
  hidden proxy textarea and keep the document in their own model + virtualized DOM,
  so a content script can't read it; *canvas* editors (Google Docs) render text to
  a `<canvas>`. In both cases the popup now declines with a clear message instead
  of issuing a credential bound to empty or partial text. Reading these would need
  an editor-specific adapter injected into the page's main world (future work).
- **Out of reach here:** native desktop apps (Word, TextEdit) — use the macOS
  capture tool, which records the same way and writes the matching document file
  alongside the credential.

## Try it

1. **Build the host:** `cargo build -p humanshipd-host`
2. **Load the extension:** open `chrome://extensions`, turn on *Developer mode*,
   click *Load unpacked*, and select this `extension/` folder. Copy the
   extension's ID that appears.
3. **Register the host** with that ID:
   `bash extension/host/install.sh <EXTENSION_ID>` then fully quit and reopen Chrome.
4. **Use it:** open any page with a text box, type a few sentences, then click the
   extension icon → *Issue Human Authored credential*. Two files land in your
   Downloads: `humanshipd-credential.c2pa` and `humanshipd-document.txt` (the exact
   text the credential is bound to — so there's nothing to reconstruct by hand).
5. **Verify it** by dropping both files into the verify page
   (<https://flagrare.github.io/humanshipd/>), or from the CLI:
   `cargo run --example verify_credential -- humanshipd-credential.c2pa humanshipd-document.txt`

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

