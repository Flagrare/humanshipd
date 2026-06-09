// Minimal STORED (uncompressed) zip writer — no dependencies. Used to bundle the
// credential + its bound document into a single download instead of two prompts.
// "Stored" (method 0) keeps the reader trivial: the verify page can extract entries
// without an inflate implementation.

(() => {
  function crc32(bytes) {
    let c = ~0;
    for (let i = 0; i < bytes.length; i++) {
      c ^= bytes[i];
      for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
    }
    return (~c) >>> 0;
  }

  function concat(parts) {
    let len = 0;
    for (const p of parts) len += p.length;
    const out = new Uint8Array(len);
    let o = 0;
    for (const p of parts) {
      out.set(p, o);
      o += p.length;
    }
    return out;
  }

  // files: [{ name: string, bytes: Uint8Array }] → a stored-zip Uint8Array.
  function makeZip(files) {
    const enc = new TextEncoder();
    const local = [];
    const central = [];
    let offset = 0;

    for (const f of files) {
      const name = enc.encode(f.name);
      const crc = crc32(f.bytes);
      const size = f.bytes.length;

      const lh = new DataView(new ArrayBuffer(30));
      lh.setUint32(0, 0x04034b50, true); // local file header signature
      lh.setUint16(4, 20, true); // version needed
      lh.setUint16(8, 0, true); // method 0 = stored
      lh.setUint32(14, crc, true);
      lh.setUint32(18, size, true);
      lh.setUint32(22, size, true);
      lh.setUint16(26, name.length, true);
      local.push(new Uint8Array(lh.buffer), name, f.bytes);

      const ch = new DataView(new ArrayBuffer(46));
      ch.setUint32(0, 0x02014b50, true); // central directory header signature
      ch.setUint16(4, 20, true); // version made by
      ch.setUint16(6, 20, true); // version needed
      ch.setUint16(10, 0, true); // method stored
      ch.setUint32(16, crc, true);
      ch.setUint32(20, size, true);
      ch.setUint32(24, size, true);
      ch.setUint16(28, name.length, true);
      ch.setUint32(42, offset, true); // local header offset
      central.push(new Uint8Array(ch.buffer), name);

      offset += 30 + name.length + size;
    }

    const centralBytes = concat(central);
    const eocd = new DataView(new ArrayBuffer(22));
    eocd.setUint32(0, 0x06054b50, true); // end of central directory signature
    eocd.setUint16(8, files.length, true); // entries on this disk
    eocd.setUint16(10, files.length, true); // total entries
    eocd.setUint32(12, centralBytes.length, true);
    eocd.setUint32(16, offset, true); // central directory offset
    return concat([concat(local), centralBytes, new Uint8Array(eocd.buffer)]);
  }

  function bytesToBase64(bytes) {
    let binary = "";
    const chunk = 0x8000;
    for (let i = 0; i < bytes.length; i += chunk) {
      binary += String.fromCharCode.apply(null, bytes.subarray(i, i + chunk));
    }
    return btoa(binary);
  }

  window.humanshipdZip = { makeZip, bytesToBase64 };
})();
