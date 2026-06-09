// Regression for the in-browser credential verifier, codifying the smoke-test
// trajectory: the page loads and initializes the WASM engine, the bundled demo
// reads as an exact-file match, a .docx of the same writing verifies as a content
// match (Decision 4's cross-format path, in the browser), an unrelated document is
// rejected, and a PDF is honestly directed to the CLI.
//
// Run: cd web-verify/tests && npm i && npx playwright install chromium && npm test
// (The Playwright config builds the WASM bundle and serves web-verify/ for these.)

const { test, expect } = require("@playwright/test");
const fs = require("fs");
const path = require("path");

const EXAMPLES = path.join(__dirname, "..", "examples");
const credential = () => fs.readFileSync(path.join(EXAMPLES, "credential.c2pa"));
const documentText = () => fs.readFileSync(path.join(EXAMPLES, "document.txt"), "utf8");

// --- minimal .docx builder (stored zip, one entry) ---------------------------
// Lets the spec wrap the example document's exact text in OOXML at runtime, so
// there's no binary fixture to commit and nothing to drift from document.txt.
function crc32(buf) {
  let c = ~0;
  for (let i = 0; i < buf.length; i++) {
    c ^= buf[i];
    for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
  }
  return (~c) >>> 0;
}

function storedZip(name, data) {
  const nameBuf = Buffer.from(name);
  const crc = crc32(data);
  const local = Buffer.alloc(30);
  local.writeUInt32LE(0x04034b50, 0); // local file header signature
  local.writeUInt16LE(20, 4); // version needed
  local.writeUInt16LE(0, 8); // method 0 = stored (no codec needed to read)
  local.writeUInt32LE(crc, 14);
  local.writeUInt32LE(data.length, 18);
  local.writeUInt32LE(data.length, 22);
  local.writeUInt16LE(nameBuf.length, 26);
  const localBlock = Buffer.concat([local, nameBuf, data]);

  const central = Buffer.alloc(46);
  central.writeUInt32LE(0x02014b50, 0); // central directory header signature
  central.writeUInt16LE(20, 4); // version made by
  central.writeUInt16LE(20, 6); // version needed
  central.writeUInt16LE(0, 10); // method = stored
  central.writeUInt32LE(crc, 16);
  central.writeUInt32LE(data.length, 20);
  central.writeUInt32LE(data.length, 24);
  central.writeUInt16LE(nameBuf.length, 28);
  central.writeUInt32LE(0, 42); // local header offset (this is the first entry)
  const cd = Buffer.concat([central, nameBuf]);

  const eocd = Buffer.alloc(22);
  eocd.writeUInt32LE(0x06054b50, 0); // end of central directory signature
  eocd.writeUInt16LE(1, 8); // entries on this disk
  eocd.writeUInt16LE(1, 10); // total entries
  eocd.writeUInt32LE(cd.length, 12); // central directory size
  eocd.writeUInt32LE(localBlock.length, 16); // central directory offset
  return Buffer.concat([localBlock, cd, eocd]);
}

function makeDocx(text) {
  const esc = text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  const xml =
    `<?xml version="1.0"?><w:document xmlns:w="x"><w:body><w:p><w:r>` +
    `<w:t>${esc}</w:t></w:r></w:p></w:body></w:document>`;
  return storedZip("word/document.xml", Buffer.from(xml, "utf8"));
}

// Multi-entry stored zip — mirrors the extension's bundle format (extension/zip.js)
// so this test exercises the verify page's reader against the real writer's output.
function bundleZip(files) {
  const local = [];
  const central = [];
  let offset = 0;
  for (const f of files) {
    const name = Buffer.from(f.name);
    const crc = crc32(f.bytes);
    const lh = Buffer.alloc(30);
    lh.writeUInt32LE(0x04034b50, 0);
    lh.writeUInt16LE(20, 4);
    lh.writeUInt16LE(0, 8); // stored
    lh.writeUInt32LE(crc, 14);
    lh.writeUInt32LE(f.bytes.length, 18);
    lh.writeUInt32LE(f.bytes.length, 22);
    lh.writeUInt16LE(name.length, 26);
    local.push(lh, name, f.bytes);
    const ch = Buffer.alloc(46);
    ch.writeUInt32LE(0x02014b50, 0);
    ch.writeUInt16LE(20, 4);
    ch.writeUInt16LE(20, 6);
    ch.writeUInt16LE(0, 10); // stored
    ch.writeUInt32LE(crc, 16);
    ch.writeUInt32LE(f.bytes.length, 20);
    ch.writeUInt32LE(f.bytes.length, 24);
    ch.writeUInt16LE(name.length, 28);
    ch.writeUInt32LE(offset, 42);
    central.push(ch, name);
    offset += 30 + name.length + f.bytes.length;
  }
  const cd = Buffer.concat(central);
  const eocd = Buffer.alloc(22);
  eocd.writeUInt32LE(0x06054b50, 0);
  eocd.writeUInt16LE(files.length, 8);
  eocd.writeUInt16LE(files.length, 10);
  eocd.writeUInt32LE(cd.length, 12);
  eocd.writeUInt32LE(offset, 16);
  return Buffer.concat([...local, cd, eocd]);
}

async function verify(page, credBuf, doc) {
  await page.goto("/verify.html");
  await expect(page.getByRole("button", { name: "Verify" })).toBeEnabled();
  await page
    .locator("#credential")
    .setInputFiles({ name: "credential.c2pa", mimeType: "application/octet-stream", buffer: credBuf });
  await page.locator("#document").setInputFiles(doc);
  await page.getByRole("button", { name: "Verify" }).click();
}

test("loads and initializes the WASM verifier", async ({ page }) => {
  await page.goto("/verify.html");
  await expect(page.getByRole("button", { name: "Verify" })).toBeEnabled();
});

test("the bundled demo reads as an exact-file match", async ({ page }) => {
  await page.goto("/verify.html");
  await expect(page.getByRole("button", { name: "Verify" })).toBeEnabled();
  await page.getByRole("button", { name: "Load a demo credential" }).click();
  await expect(page.locator("#verdict")).toContainText("exact file");
  await expect(page.locator("#result")).toHaveClass(/valid/);
  // Decision 6: the trust framing must be honest about the self-signed default.
  await expect(page.locator("#trust")).toContainText("self-signed");
  await expect(page.locator("#trust")).toContainText("not who wrote it");
});

test("issues a credential in-browser (WASM) and it verifies as an exact file", async ({ page }) => {
  // De-risk spike for in-browser issuance: ephemeral key-gen + c2pa signing must run
  // in WASM, then read back as a valid exact-file match — all without a native host.
  await page.goto("/verify.html");
  await expect(page.getByRole("button", { name: "Verify" })).toBeEnabled();
  const out = await page.evaluate(async () => {
    const text = "Provenance beats inference. We bind a credential to the writing process.";
    const session = JSON.stringify({
      session_id: "wasm-spike",
      surface_kind: "gdocs",
      surface_app: "docs.google.com",
      final_text: text,
      events: [{ at_ms: 0, inserted_chars: text.length, deleted_chars: 0, keystrokes: text.length }],
      author: "Ada",
    });
    const manifest = window.issue_credential(session); // Uint8Array, signed in WASM
    const doc = new TextEncoder().encode(text);
    const r = window.verify_credential_named(manifest, doc, "doc.txt");
    return { len: manifest.length, valid: r.valid, tier: r.verdict && r.verdict.tier, signed: r.trust && r.trust.signed };
  });
  expect(out.len).toBeGreaterThan(1000);
  expect(out.valid).toBe(true);
  expect(out.tier).toBe("exact_file");
  expect(out.signed).toBe(true);
});

test("a single .zip bundle (credential + document) verifies as an exact file", async ({ page }) => {
  await page.goto("/verify.html");
  await expect(page.getByRole("button", { name: "Verify" })).toBeEnabled();
  const zip = bundleZip([
    { name: "humanshipd-credential.c2pa", bytes: credential() },
    { name: "humanshipd-document.txt", bytes: Buffer.from(documentText(), "utf8") },
  ]);
  await page
    .locator("#credential")
    .setInputFiles({ name: "humanshipd-credential.zip", mimeType: "application/zip", buffer: zip });
  await page.getByRole("button", { name: "Verify" }).click();
  await expect(page.locator("#verdict")).toContainText("exact file");
  await expect(page.locator("#result")).toHaveClass(/valid/);
});

test("a .docx of the same writing verifies as a content match", async ({ page }) => {
  await verify(page, credential(), {
    name: "essay.docx",
    mimeType: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    buffer: makeDocx(documentText()),
  });
  await expect(page.locator("#verdict")).toContainText(/same content|same writing/);
  await expect(page.locator("#result")).toHaveClass(/valid/);
});

test("an unrelated document does not match", async ({ page }) => {
  await verify(page, credential(), {
    name: "unrelated.txt",
    mimeType: "text/plain",
    buffer: Buffer.from(
      "Tide charts and coral spawning cycles govern when the reef releases its gametes " +
        "across the atoll each year, tracked by lunar phase and water temperature."
    ),
  });
  await expect(page.locator("#verdict")).toContainText("No match");
  await expect(page.locator("#result")).toHaveClass(/invalid/);
});

test("a PDF is honestly directed to the command-line tool", async ({ page }) => {
  await verify(page, credential(), {
    name: "essay.pdf",
    mimeType: "application/pdf",
    buffer: Buffer.from("%PDF-1.4\n% not a real pdf\n"),
  });
  await expect(page.locator("#verdict")).toContainText("Could not verify");
  await expect(page.locator("#verdict")).toContainText("command-line tool");
});
