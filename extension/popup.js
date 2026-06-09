// Popup: grab the current writing session from the active tab, sign a credential
// in-browser (WASM, in a worker), and download it bundled with its document.

const button = document.getElementById("issue");
const status = document.getElementById("status");

function show(text, cls) {
  status.textContent = text;
  status.className = cls || "";
}

// Sign the credential in a Web Worker running the WASM core — off the popup thread,
// and with no native host to install. Resolves to the .c2pa manifest Uint8Array.
function issueViaWorker(session) {
  return new Promise((resolve, reject) => {
    const worker = new Worker(chrome.runtime.getURL("issue-worker.js"), { type: "module" });
    worker.onmessage = (event) => {
      worker.terminate();
      if (event.data?.ok) resolve(event.data);
      else reject(new Error(event.data?.error || "issue failed"));
    };
    worker.onerror = (event) => {
      worker.terminate();
      reject(new Error(event.message || "worker error"));
    };
    worker.postMessage(session);
  });
}

button.addEventListener("click", async () => {
  show("Collecting writing session…");
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tab?.id) {
    show("No active tab.", "err");
    return;
  }

  let session;
  try {
    session = await chrome.tabs.sendMessage(tab.id, { type: "getSession" });
  } catch {
    show("Could not reach the page. Open a web editor and type first.", "err");
    return;
  }
  if (!session || (!session.log && !session.events?.length)) {
    show("No writing captured yet — type in a text field, then try again.", "err");
    return;
  }
  if (!session.log) {
    if (session.code_editor) {
      show(
        "This looks like a code editor (Ace / CodeMirror / Monaco). humanshipd can't read its text yet — its content isn't in the page. Try a plain text box, or the macOS app for desktop editors.",
        "err"
      );
      return;
    }
    if (!session.final_text || !session.final_text.trim()) {
      show(
        "Couldn't read any text from this editor — it may be canvas- or model-based. Try a plain text box, or the macOS app.",
        "err"
      );
      return;
    }
  }

  show("Signing credential in your browser…");
  const author = document.getElementById("author").value.trim();
  let manifestBytes;
  let documentText;
  try {
    const payload = session.log
      ? { log: session.log, author }
      : { session: { ...session, author }, author };
    const result = await issueViaWorker(payload);
    manifestBytes = result.manifest;
    documentText = session.log ? result.text : session.final_text;
  } catch (e) {
    show(`Could not issue: ${e.message}`, "err");
    return;
  }

  // Bundle the credential AND the exact document text it's bound to into ONE zip,
  // so it's a single download (not two prompts) and a single drop on the verify
  // page. The document text is needed to reproduce the exact bytes the credential
  // is hash-bound to — especially for Google Docs, whose text lives only in the
  // cloud. The credential itself stays content-free; the text rides alongside it.
  const docBytes = new TextEncoder().encode(documentText);
  const zipBytes = humanshipdZip.makeZip([
    { name: "humanshipd-credential.c2pa", bytes: manifestBytes },
    { name: "humanshipd-document.txt", bytes: docBytes },
  ]);
  await chrome.downloads.download({
    url: `data:application/zip;base64,${humanshipdZip.bytesToBase64(zipBytes)}`,
    filename: "humanshipd-credential.zip",
    saveAs: false,
  });
  show(
    "Saved humanshipd-credential.zip to your Downloads. Drop it into the verify page.",
    "ok"
  );
});
