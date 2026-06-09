// Injected script (MAIN world, docs.google.com, document_start): Google Docs talks
// to its backend over private `/save` requests, invisible to an isolated content
// script. This runs in the page's OWN JS context to observe those requests, pulls
// the mutation command ops out of each save body, and forwards them to gdocs.js
// (the isolated world) via window.postMessage.
//
// It reads only what the page already sends to Google, and forwards only the
// content-free mutation ops (positions, text, timing) to our own extension — never
// to any third party. Registered as a manifest content script with "world":"MAIN"
// so no inline <script> injection is needed.

(() => {
  const TAG = "humanshipd-gdocs";

  const isSave = (url) => {
    try {
      return String(url).includes("/save");
    } catch (_) {
      return false;
    }
  };

  // Pull the mutation command ops out of a Docs save body. Commands live under a
  // `bundles` parameter — a JSON array of `{ commands: [op, …] }`, where each op is
  // `{ ty: "is"|"ds"|"mlti", … }` (the same shape core/src/gdocs.rs parses).
  // Tolerant of both form-encoded (`bundles=<urlencoded JSON>`) and raw-JSON bodies.
  function extractOps(body) {
    const json = bundlesJson(body);
    if (!json) return [];
    let bundles;
    try {
      bundles = JSON.parse(json);
    } catch (_) {
      return [];
    }
    if (!Array.isArray(bundles)) bundles = [bundles];
    const ops = [];
    for (const bundle of bundles) {
      const commands = (bundle && bundle.commands) || [];
      for (const command of commands) ops.push(command);
    }
    return ops;
  }

  function bundlesJson(body) {
    if (typeof body !== "string" || body.length === 0) return null;
    const form = body.match(/(?:^|&)bundles=([^&]*)/);
    if (form) {
      try {
        return decodeURIComponent(form[1]);
      } catch (_) {
        return form[1];
      }
    }
    if (body.indexOf('"bundles"') !== -1) {
      try {
        return JSON.stringify(JSON.parse(body).bundles);
      } catch (_) {
        return null;
      }
    }
    return null;
  }

  function forward(body) {
    const at = Date.now();
    for (const op of extractOps(body)) {
      window.postMessage({ source: TAG, kind: "op", op, at }, "*");
    }
  }

  const origFetch = window.fetch;
  if (typeof origFetch === "function") {
    window.fetch = function (input, init) {
      try {
        const url = typeof input === "string" ? input : input && input.url;
        if (isSave(url) && init && typeof init.body === "string") forward(init.body);
      } catch (_) {
        /* never break the page's own request */
      }
      return origFetch.apply(this, arguments);
    };
  }

  const origOpen = XMLHttpRequest.prototype.open;
  const origSend = XMLHttpRequest.prototype.send;
  XMLHttpRequest.prototype.open = function (_method, url) {
    this.__humanshipdSave = isSave(url);
    return origOpen.apply(this, arguments);
  };
  XMLHttpRequest.prototype.send = function (body) {
    try {
      if (this.__humanshipdSave && typeof body === "string") forward(body);
    } catch (_) {
      /* never break the page's own request */
    }
    return origSend.apply(this, arguments);
  };
})();
