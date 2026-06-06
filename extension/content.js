// Content script: observe how text is written in a web editor (textarea, input,
// or contenteditable) and build a metadata-only event stream. Distinguishes
// typed input from pasted input. The captured text is only ever sent to the
// LOCAL humanshipd host (for hashing); it is not transmitted anywhere else.
//
// Note: Google Docs renders to <canvas>, so its text isn't readable here — this
// POC adapter targets ordinary web editors. Docs needs its own (Draftback-style)
// capture path, which is future work.

(() => {
  let startTime = null;
  let events = [];
  let lastText = "";
  let pastePending = false;

  function editableText(el) {
    if (!el) return null;
    const tag = el.tagName;
    if (tag === "TEXTAREA" || tag === "INPUT") return el.value;
    if (el.isContentEditable) return el.innerText;
    return null;
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
  chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    if (message?.type === "getSession") {
      sendResponse({
        session_id: `web-${Date.now()}`,
        surface_kind: "web",
        surface_app: location.hostname || "web",
        final_text: lastText,
        events,
      });
    }
    return true;
  });
})();
