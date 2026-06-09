// Content script (ISOLATED world, docs.google.com): turn Google Docs' live edit
// stream into a content-free writing session. It receives `/save` mutation ops from
// gdocs-inject.js (which runs in the page's own JS context — see that file) and
// `paste` clipboard events, replays them into a reconstructed document + event
// stream, and answers the popup's `getSession`.
//
// Replay mirrors core/src/gdocs.rs::apply_op so the live path and the historical
// /revisions/load path reconstruct byte-identical text (and thus the same content
// fingerprint). Only counts, timing, and the locally-reconstructed text exist here;
// nothing is sent anywhere except the local host at issue time.
//
// Paste detection (Decision 1): the `paste` event is the real paste signal. It
// fires in the editor *iframe*, while `/save` fires from the top page — so every
// frame captures paste, subframes forward to the top frame, and the top frame is
// the single aggregator that merges ops + pastes and answers the popup.

(() => {
  const TAG = "humanshipd-gdocs";
  const isTop = window.top === window;

  let startTime = null;
  const buf = []; // reconstruction buffer, one entry per code point
  const events = [];
  let pendingPaste = null; // { len, at } awaiting the next insert to flag as a paste

  const charsOf = (s) => Array.from(s || "");

  function stamp(at) {
    if (startTime === null) startTime = at;
    return Math.max(at - startTime, 0);
  }

  // A paste flags the next insert of matching length within a short window. Docs
  // coalesces a paste into one `is`, so size + timing line up (validated 2026-06-07).
  function consumePasteFor(len, at) {
    if (!pendingPaste) return false;
    const recent = at - pendingPaste.at < 4000;
    const sized = pendingPaste.len === 0 || pendingPaste.len === len;
    if (recent && sized) {
      pendingPaste = null;
      return true;
    }
    return false;
  }

  // Replay one op into buf + events, recursing `mlti` bundles. Mirrors gdocs.rs.
  function applyOp(op, at) {
    switch (op && op.ty) {
      case "is": {
        const ibi = Number.isInteger(op.ibi) ? op.ibi : 1;
        const chars = charsOf(op.s);
        if (chars.length === 0) return;
        const pos = Math.min(Math.max(ibi - 1, 0), buf.length);
        buf.splice(pos, 0, ...chars);
        const pasted = consumePasteFor(chars.length, at);
        events.push({
          at_ms: stamp(at),
          inserted_chars: chars.length,
          deleted_chars: 0,
          keystrokes: pasted ? 0 : chars.length,
          at_offset: Math.max(ibi - 1, 0),
        });
        break;
      }
      case "ds": {
        const si = Number.isInteger(op.si) ? op.si : 1;
        const ei = Number.isInteger(op.ei) ? op.ei : si;
        const start = Math.min(Math.max(si - 1, 0), buf.length);
        const end = Math.max(Math.min(ei, buf.length), start); // ei is 1-based inclusive
        const n = end - start;
        if (n === 0) return;
        buf.splice(start, n);
        events.push({
          at_ms: stamp(at),
          inserted_chars: 0,
          deleted_chars: n,
          keystrokes: n,
          at_offset: Math.max(si - 1, 0),
        });
        break;
      }
      case "mlti": {
        const mts = Array.isArray(op.mts) ? op.mts : [];
        for (const sub of mts) applyOp(sub, at);
        break;
      }
      default:
        break; // style/setup ops carry no text
    }
  }

  window.addEventListener("message", (event) => {
    const d = event.data;
    if (!d || d.source !== TAG) return;
    if (d.kind === "op" && isTop) applyOp(d.op, d.at);
    else if (d.kind === "paste" && isTop) pendingPaste = { len: d.len, at: d.at };
  });

  // Capture paste in every frame; forward to the top aggregator.
  document.addEventListener(
    "paste",
    (event) => {
      let len = 0;
      try {
        len = charsOf(event.clipboardData && event.clipboardData.getData("text/plain")).length;
      } catch (_) {
        len = 0;
      }
      const msg = { source: TAG, kind: "paste", len, at: Date.now() };
      (isTop ? window : window.top).postMessage(msg, "*");
    },
    true
  );

  if (!isTop) return; // subframes only forward paste; the top frame owns the session

  chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    if (message?.type !== "getSession") return false;
    if (events.length === 0) return false; // nothing captured here — stay silent
    sendResponse({
      session_id: `gdocs-${Date.now()}`,
      surface_kind: "gdocs",
      surface_app: "docs.google.com",
      final_text: buf.join(""),
      events,
    });
    return true;
  });
})();
