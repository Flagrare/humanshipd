# Design: Content Binding & Capture Fidelity

- **Date:** 2026-06-06
- **Status:** draft (awaiting user review)
- **Companion to:** [`2026-06-05-human-authorship-attestation-design.md`](./2026-06-05-human-authorship-attestation-design.md) (architecture, threat model, credential) and [`2026-06-06-authorship-signals-and-reporting-design.md`](./2026-06-06-authorship-signals-and-reporting-design.md) (signals, report).
- **Motivating incidents (2026-06-06):** a credential issued from texteditor.co verified as INVALID; the downloaded document came back empty. Root causes below.

## 1. Summary

A credential is only useful if a reader can take an everyday document and confirm "this is the writing the credential vouches for." Two gaps stand between us and that, and they're easy to conflate:

1. **Capture fidelity** — *can we read the text the person actually wrote?* Many web editors don't expose their text to a content script at all.
2. **Binding fidelity** — *does the thing the reader uploads match what the credential bound to?* The credential binds to the exact bytes of one plain-text serialization, but a real document is a formatted container.

Both are facets of one principle: **the credential should vouch for the *content a human wrote*, not for the bytes of a particular editor or file format.** Today it does neither cleanly, and this spec is how we close the gap honestly — including being explicit, in the product, about where we can't.

## 2. The two gaps, concretely

### Capture fidelity (why the document came back empty)

texteditor.co runs the **Ace** code editor. Ace — like **CodeMirror** and **Monaco** — types into a *hidden proxy `<textarea>`* and keeps the real document in its own model, rendering it into **virtualized DOM** (off-screen lines aren't even present). So `textarea.value` is a tiny input buffer, not the document. Our generic content script read that buffer → "3 characters" → an empty file.

Web editors fall into capture classes:

- **Readable:** real `<textarea>`/`<input>`, and rich `contenteditable` editors (tiptap, ProseMirror, Quill) — including inside `about:blank`/`designMode` iframes. *(Now captured correctly; honest refusal shipped for the rest — commit `c835e13`.)*
- **Unreadable by a content script:** code editors (Ace/CodeMirror/Monaco — proxy textarea + virtualized DOM) and canvas editors (Google Docs). Their text simply isn't in the page DOM.

### Binding fidelity (why the .rtf was INVALID, and the deeper point)

The credential binds with `sha256(exact plain-text bytes)` and the verifier hashes the uploaded file's raw bytes. So the binding is to *one exact serialization*. But the same writing saved as `.rtf`, `.docx`, `.pdf`, or EPUB has entirely different bytes — and **"very rarely is a document pure text."** Even a perfect re-save in a rich format can never match. The credential vouches for the *writing*; the container format is incidental, yet the binding is glued to it.

## 3. Binding fidelity: bind to content, verify by extraction

The credential already carries the right ingredient and ignores it. Every credential stores an **ISCC soft binding** — a similarity-preserving content code over the text (exactly C2PA's "Durable Content Credentials" mechanism) — but `read_sidecar` only checks the hard hash. That's the lever.

**Two tiers, and the tradeoff between them is the whole design:**

| Tier | Statement | Strength | Cost |
|---|---|---|---|
| **Hard binding** (today) | "This is *that exact file*, untampered." | Cryptographically crisp | Brittle — any reformatting breaks it |
| **Content binding** (ISCC, already stored) | "This is the *same writing*, possibly reformatted or lightly edited." | Robust across formats | Fuzzy — a similarity threshold, not yes/no |

**Content-aware verification.** On verify, extract the text from the uploaded document, then report *both* tiers:

1. **Extract** text by format, easiest/most-common first:
   - `text/plain`, `.md` — bytes as UTF-8 (today's path).
   - `.docx` — unzip, read `word/document.xml`, concatenate text runs with paragraph breaks. (Pure-Rust zip+XML; WASM-friendly.)
   - `.rtf` — strip control words/groups to plain text.
   - `.pdf` — text extraction is the hard one (layout, encodings); a later phase, likely via a dedicated extractor.
   - Unknown/binary — fall back to raw-bytes hard match and say "couldn't read this format's text."
2. **Match:** exact (`sha256` of the extracted text vs. the hard binding, plus the existing raw-bytes exact match) and content (ISCC of the extracted text vs. the soft binding, within a similarity threshold).
3. **Report honestly** — three outcomes, never collapsed into one:
   - **Exact match** — the document the credential was issued for.
   - **Content match** — the same writing in a different wrapper / lightly edited (show it's not byte-identical, and roughly how close).
   - **No match** — the text doesn't correspond to this credential.

**The honesty gradient (state it in the UI):** raw-bytes → extracted-text → ISCC-similarity each trade crispness for robustness. Only the hard binding is a cryptographic "this exact document." Content match is evidence, not proof, and needs a published threshold (too loose → false matches). The verify page must label which tier produced the verdict, the way the report already separates provenance from inference.

**Alternative / long-term — embed the manifest in the document.** C2PA supports embedding manifests into Office, PDF, and EPUB. Then the credential travels *inside* the file and hard-binds to the file's own bytes — the canonical "Content Credentials" model, no separate document to match. It requires a "seal" step that operates on the final exported file (we capture the process while writing, but don't produce the `.docx`), and edits after sealing fall back to the soft binding / registry recovery. Worth doing, but it's a workflow addition, not a swap.

## 4. Capture fidelity: read the editor, or say you can't

- **Shipped (honest refusal):** detect code/canvas editors and decline with a clear message rather than issue a credential over empty/partial text. Capture the full `contenteditable` host and inject into `about:blank` iframes so the editors we *can* read, we read fully.
- **Main-world editor adapters (future):** a content script runs in an isolated world and can't call the page's `ace`/CodeMirror/Monaco instance. A small injected main-world shim could read `editor.getValue()` (Ace), `view.state.doc` (CodeMirror 6), or the Monaco model — turning "unreadable" into "readable" per editor, behind explicit detection. Each is an adapter, opt-in and named, never a guess.
- **Google Docs** remains its own path (canvas; the Draftback approach of reading the in-page revision model).
- **Why the desktop adapter matters here:** the macOS Accessibility adapter reads the *rendered* text the OS exposes, independent of an editor's internal model — so for Word, Scrivener, and many editors it sidesteps this entire class of problem. For serious authoring, desktop capture is often *more* reliable than the browser, not less.

## 5. Unifying principle

Organize both capture and binding around **content, with explicit coverage**. Capture reads the writing where it can and refuses honestly where it can't; binding vouches for the writing (hard when byte-identical, content-similar otherwise) rather than for a container's bytes. Every gap — this editor, that format — is disclosed in the product, not discovered by a confused user.

## 6. Roadmap

**Phase A — content-aware verification (highest user value).**
- Wire the stored ISCC soft binding into `read_sidecar`; return a tiered verdict (exact / content / none).
- Text extraction for `.txt`/`.md`, then `.docx`, then `.rtf`. Surface the tier + similarity on the verify page with the honesty-gradient framing.

**Phase B — capture adapters for readable-but-modeled editors.**
- Main-world adapters for Ace / CodeMirror / Monaco behind explicit detection (replacing today's refusal for those).

**Phase C — in-document sealing.**
- A "seal this file" flow that embeds the C2PA manifest into `.docx`/`.pdf`/EPUB, so the credential travels inside the document and verifying is just opening the file.

**Phase D — `.pdf` text extraction** and Google Docs in-page capture.

## 7. Open questions

- **Extraction ambiguity:** which text counts in a `.docx`/`.pdf`? Headers/footers, footnotes, tracked-changes, alt text — include or exclude? The answer must be deterministic so the same document always extracts the same text.
- **ISCC threshold:** what Hamming distance counts as "same writing"? Too loose invites false matches; publish the number and its measured false-match rate.
- **Normalization:** newline/whitespace canonicalization for the extracted-text tier — necessary for cross-format matching, but it weakens "exact," so keep the raw-bytes tier distinct.
- **Main-world injection:** reading a page's editor instance means running code in the page world; scope it tightly and document the trust implication.
- Does sealing (Phase C) belong in the browser extension, the desktop app, or a CLI step?
