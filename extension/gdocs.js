// Content script (ISOLATED world, docs.google.com): records normalized writing ops
// from Google Docs' live edit stream without reconstructing text. Receives `/save`
// mutation ops forwarded by gdocs-inject.js (page JS context) and `paste` clipboard
// events, converts them into CapturedOps (insert/delete), and persists each
// page-load as a sharded session in chrome.storage.local. On reload it resumes from
// prior sessions so typing across multiple visits accumulates into one log.
// Text reconstruction and stat aggregation now live in core's `build_record`, not here.
//
// Paste detection (Decision 1): the `paste` event is the real paste signal. It
// fires in the editor *iframe*, while `/save` fires from the top page — so every
// frame captures paste, subframes forward to the top frame, and the top frame is
// the single aggregator that merges ops + pastes and answers the popup.

(() => {
  const TAG = "humanshipd-gdocs";
  const isTop = window.top === window;

  const docId = (location.pathname.match(/\/d\/([^/]+)/) || [])[1] || "unknown";
  const KEY_PREFIX = `humanshipd:log:gdocs:${docId}`;

  let priorSessions = []; // sessions loaded from storage (earlier page-loads)
  let sessionIndex = 0; // this page-load's session number
  let loaded = false; // true once prior sessions are read and sessionIndex is final
  let startedAtMs = Date.now();
  const ops = []; // this session's normalized CapturedOps
  let pendingPaste = null; // { len, at } awaiting the next insert

  const charsOf = (s) => Array.from(s || "");

  function consumePasteFor(len, at) {
    if (!pendingPaste) return false;
    const recent = at - pendingPaste.at < 4000;
    const sized = pendingPaste.len === 0 || pendingPaste.len === len;
    if (recent && sized) { pendingPaste = null; return true; }
    return false;
  }

  // Translate a Google Docs save op (is/ds/mlti) into normalized CapturedOps and
  // record them. Text reconstruction now lives in core (build_record), not here.
  function recordOp(op, at) {
    switch (op && op.ty) {
      case "is": {
        const ibi = Number.isInteger(op.ibi) ? op.ibi : 1;
        const chars = charsOf(op.s);
        if (chars.length === 0) return;
        const pasted = consumePasteFor(chars.length, at);
        ops.push({ op: "insert", at_ms: Math.max(at - startedAtMs, 0), pos: Math.max(ibi - 1, 0), text: chars.join(""), pasted });
        break;
      }
      case "ds": {
        const si = Number.isInteger(op.si) ? op.si : 1;
        const ei = Number.isInteger(op.ei) ? op.ei : si;
        const len = Math.max(ei - si + 1, 0);
        if (len === 0) return;
        ops.push({ op: "delete", at_ms: Math.max(at - startedAtMs, 0), pos: Math.max(si - 1, 0), len });
        break;
      }
      case "mlti": {
        for (const sub of (op.mts || [])) recordOp(sub, at);
        break;
      }
      default:
        break;
    }
    saveSoon();
  }

  function currentSession() {
    return {
      session_id: `gdocs-${docId}-${sessionIndex}`,
      surface_kind: "gdocs",
      surface_app: "docs.google.com",
      started_at_ms: startedAtMs,
      ops,
    };
  }

  let saveTimer = null;
  function saveSoon() {
    if (saveTimer) return;
    saveTimer = setTimeout(saveNow, 1500);
  }
  function saveNow() {
    saveTimer = null;
    // Don't write until the resume-load has set the final sessionIndex — otherwise an
    // early keystroke could persist under s0 and clobber a prior session's key.
    if (!loaded || ops.length === 0) return;
    chrome.storage.local.set({ [`${KEY_PREFIX}:s${sessionIndex}`]: currentSession() });
  }
  window.addEventListener("pagehide", saveNow, true);
  document.addEventListener("visibilitychange", () => { if (document.visibilityState === "hidden") saveNow(); }, true);

  // On load (top frame only): pull prior sessions, start a new session after them.
  if (isTop) {
    chrome.storage.local.get(null, (all) => {
      const keys = Object.keys(all || {})
        .filter((k) => k.startsWith(`${KEY_PREFIX}:s`))
        .sort((a, b) => Number(a.split(":s")[1]) - Number(b.split(":s")[1]));
      priorSessions = keys.map((k) => all[k]);
      sessionIndex = priorSessions.length; // this page-load is the next session
      loaded = true;
      if (ops.length > 0) saveSoon(); // flush anything typed while the load was in flight
    });
  }

  window.addEventListener("message", (event) => {
    const d = event.data;
    if (!d || d.source !== TAG) return;
    if (d.kind === "op" && isTop) recordOp(d.op, d.at);
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
    const sessions = ops.length > 0 ? [...priorSessions, currentSession()] : [...priorSessions];
    if (sessions.length === 0 || sessions.every((s) => s.ops.length === 0)) return false;
    sendResponse({
      log: { schema: "authorshipped/log@1", identity: { kind: "gdocs", id: docId }, sessions },
      surface_kind: "gdocs",
    });
    return true;
  });
})();
