use std::io::{self, Read, Write};

/// Upper bound on an accepted frame length. Real issue requests are kilobytes; this
/// cap bounds a corrupt or hostile length prefix so it can never trigger a
/// multi-gigabyte allocation (which on a constrained machine causes system-wide
/// memory pressure — and the OS may then kill the largest process, e.g. the browser).
const MAX_FRAME_LEN: usize = 64 * 1024 * 1024; // 64 MiB

/// Read one Native Messaging frame: a 4-byte little-endian length prefix
/// followed by that many UTF-8 JSON bytes. Returns `Ok(None)` at clean EOF.
pub fn read_message<R: Read>(reader: &mut R) -> io::Result<Option<Vec<u8>>> {
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let len = u32::from_le_bytes(len_buf) as usize;
    if len > MAX_FRAME_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("native message frame length {len} exceeds the {MAX_FRAME_LEN}-byte limit"),
        ));
    }
    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload)?;
    Ok(Some(payload))
}

/// Write one Native Messaging frame (4-byte little-endian length + payload) and flush.
pub fn write_message<W: Write>(writer: &mut W, payload: &[u8]) -> io::Result<()> {
    let len = payload.len() as u32;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(payload)?;
    writer.flush()
}
