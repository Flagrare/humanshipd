// Content script: observe how text is written in a web editor (textarea, input,
// or contenteditable) and build a metadata-only event stream. Distinguishes
// typed input from pasted input. The captured text is only ever sent to the
// LOCAL humanshipd host (for hashing); it is not transmitted anywhere else.
//
// Note: Google Docs renders to <canvas>, so its text isn't readable here — it has
// its own adapter (gdocs-inject.js + gdocs.js), and the manifest excludes
// docs.google.com from this generic script so the two don't both answer getSession.

(() => {
  let startTime = null;
  let events = [];
  let lastText = "";
  let pastePending = false;

  function editableText(el) {
    if (!el) return null;
    const tag = el.tagName;
    if (tag === "TEXTAREA" || tag === "INPUT") return el.value;
    if (el.isContentEditable) {
      // The input event can target a node deep inside a rich editor (tiptap,
      // ProseMirror, Quill…); read the whole editable host, not just that node.
      const host = el.closest('[contenteditable=""], [contenteditable="true"]');
      if (host) return host.innerText;
      // Old-style editors make a whole (often about:blank) iframe editable.
      const doc = el.ownerDocument;
      if (doc && doc.designMode === "on") return doc.body.innerText;
      return el.innerText;
    }
    return null;
  }

  // Code editors (Ace, CodeMirror, Monaco) type into a hidden proxy textarea and
  // keep the real document in their own model + virtualized DOM — so we can't read
  // their text from the page. Detect them so the popup can refuse honestly rather
  // than issue a credential bound to an empty or partial buffer.
  function codeEditorPresent() {
    return !!document.querySelector(
      ".ace_editor, .cm-editor, .CodeMirror, .monaco-editor"
    );
  }

  // The caret/edit offset when we can read it: for input/textarea, the start of
  // the current selection (the insertion point). Contenteditable carets aren't a
  // simple character index, so we return null there (position unknown — honest).
  function editOffset(el) {
    if (!el) return null;
    const tag = el.tagName;
    if ((tag === "TEXTAREA" || tag === "INPUT") && typeof el.selectionStart === "number") {
      return el.selectionStart;
    }
    return null;
  }

  function charLen(s) {
    return Array.from(s).length;
  }

  document.addEventListener(
    "paste",
    () => {
      pastePending = true;
    },
    true
  );

  document.addEventListener(
    "input",
    (event) => {
      const text = editableText(event.target);
      if (text === null) return;

      if (startTime === null) startTime = Date.now();
      const newLen = charLen(text);
      const delta = newLen - charLen(lastText);
      const inserted = Math.max(delta, 0);
      const deleted = Math.max(-delta, 0);
      // Typed insertions carry keystrokes; a paste arrives with none.
      const keystrokes = pastePending ? 0 : inserted;
      // The caret sits at the end of the just-inserted text; the edit started
      // `inserted` chars earlier.
      const caret = editOffset(event.target);
      const at_offset = caret === null ? null : Math.max(caret - inserted, 0);

      events.push({
        at_ms: Date.now() - startTime,
        inserted_chars: inserted,
        deleted_chars: deleted,
        keystrokes,
        at_offset,
      });

      lastText = text;
      pastePending = false;
    },
    true
  );

  // The popup asks for the current session; hand back the metadata + final text.
  // The content script runs in EVERY frame (manifest `all_frames`), and a tab
  // message reaches all of them — so a frame that captured nothing (e.g. the page
  // shell around an editor iframe) must stay silent, or its empty answer can win
  // the race and produce an empty document. Only the frame that actually recorded
  // writing responds.
  chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    if (message?.type !== "getSession") return false;
    if (events.length === 0) return false; // nothing captured here — let another frame answer
    sendResponse({
      session_id: `web-${Date.now()}`,
      surface_kind: "web",
      surface_app: location.hostname || "web",
      final_text: lastText,
      events,
      code_editor: codeEditorPresent(),
    });
    return true;
  });
})();
