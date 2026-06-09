// Popup: grab the current writing session from the active tab, ask the host to
// issue a credential, and download the resulting .c2pa file.

const button = document.getElementById("issue");
const status = document.getElementById("status");

function show(text, cls) {
  status.textContent = text;
  status.className = cls || "";
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
  if (!session || !session.events?.length) {
    show("No writing captured yet — type in a text field, then try again.", "err");
    return;
  }
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

  show("Issuing credential via local host…");
  const author = document.getElementById("author").value.trim();
  const result = await chrome.runtime.sendMessage({ type: "issue", session, author });
  if (!result?.ok) {
    show(`Host error: ${result?.error || "unknown"}`, "err");
    return;
  }

  // Bundle the credential AND the exact document text it's bound to into ONE zip,
  // so it's a single download (not two prompts) and a single drop on the verify
  // page. The document text is needed to reproduce the exact bytes the credential
  // is hash-bound to — especially for Google Docs, whose text lives only in the
  // cloud. The credential itself stays content-free; the text rides alongside it.
  const manifestBytes = Uint8Array.from(atob(result.manifest_b64), (c) => c.charCodeAt(0));
  const docBytes = new TextEncoder().encode(session.final_text);
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
