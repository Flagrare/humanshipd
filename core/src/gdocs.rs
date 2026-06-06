//! Parse a Google Docs revision-log (`/revisions/load`) response into a
//! content-free writing session, so an already-written Doc can be turned into a
//! credential. The format was confirmed live against a real authenticated doc on
//! 2026-06-06 (see `docs/research/2026-06-06-google-docs-writing-capture.md`):
//!
//! - The body starts with the anti-JSON-hijack guard `)]}'` then JSON with a
//!   `changelog` array.
//! - Each changelog entry is `[op, timestampMs, userId, revisionN, sessionId, …]`.
//! - Ops: `is` (insert: `ibi` 1-based index, `s` string), `ds` (delete: `si`/`ei`,
//!   1-based inclusive), `mlti` (bundle: `mts` sub-ops). Style ops (`as`, …) carry
//!   no text and are skipped.
//!
//! Honest limitation: the revision log *coalesces* a save's worth of typing into
//! one `is`, so it can't reliably tell fast typing from a paste — this path does
//! not flag pastes (every insert is treated as keyed). Reliable paste detection
//! for Docs needs the live `/save` stream or clipboard events (future work).

use crate::error::CoreError;
use crate::session::{EditEvent, SessionInput};
use serde_json::Value;

/// Build a content-free `SessionInput` from a raw `/revisions/load` response body.
/// The document text is reconstructed locally to compute the binding hash; it is
/// never stored in the resulting record.
pub fn session_from_changelog(
    body: &str,
    session_id: impl Into<String>,
    app: impl Into<String>,
) -> Result<SessionInput, CoreError> {
    let json = strip_xssi_prefix(body);
    let root: Value = serde_json::from_str(json)
        .map_err(|e| CoreError::Serialization(format!("gdocs changelog: {e}")))?;
    let changelog = root
        .get("changelog")
        .and_then(Value::as_array)
        .ok_or_else(|| CoreError::Serialization("gdocs changelog: missing `changelog`".into()))?;

    let mut buf: Vec<char> = Vec::new();
    let mut events: Vec<EditEvent> = Vec::new();
    let mut base_ts: Option<u64> = None;

    for entry in changelog {
        let Some(arr) = entry.as_array() else { continue };
        let Some(op) = arr.first() else { continue };
        let ts = arr.get(1).and_then(Value::as_u64);
        if let Some(t) = ts {
            base_ts.get_or_insert(t);
        }
        let at_ms = match (ts, base_ts) {
            (Some(t), Some(b)) => t.saturating_sub(b),
            _ => 0,
        };
        apply_op(op, at_ms, &mut buf, &mut events);
    }

    Ok(SessionInput {
        session_id: session_id.into(),
        surface_kind: "gdocs".into(),
        surface_app: app.into(),
        final_text: buf.into_iter().collect(),
        events,
    })
}

/// Strip Google's `)]}'` anti-JSON-hijack prefix (and any leading whitespace).
fn strip_xssi_prefix(body: &str) -> &str {
    body.trim_start()
        .strip_prefix(")]}'")
        .unwrap_or_else(|| body.trim_start())
        .trim_start()
}

/// Apply one changelog op to the reconstruction buffer and emit an `EditEvent`.
/// Recurses into `mlti` bundles; ignores non-text ops (styles, etc.).
fn apply_op(op: &Value, at_ms: u64, buf: &mut Vec<char>, events: &mut Vec<EditEvent>) {
    match op.get("ty").and_then(Value::as_str) {
        Some("is") => {
            let ibi = op.get("ibi").and_then(Value::as_u64).unwrap_or(1);
            let s = op.get("s").and_then(Value::as_str).unwrap_or_default();
            let chars: Vec<char> = s.chars().collect();
            let n = chars.len() as u64;
            if n == 0 {
                return;
            }
            let pos = ((ibi.saturating_sub(1)) as usize).min(buf.len());
            buf.splice(pos..pos, chars);
            events.push(EditEvent {
                at_ms,
                inserted_chars: n,
                deleted_chars: 0,
                keystrokes: n, // see module note: typed vs. pasted is indistinguishable here
                at_offset: Some(ibi.saturating_sub(1)),
            });
        }
        Some("ds") => {
            let si = op.get("si").and_then(Value::as_u64).unwrap_or(1);
            let ei = op.get("ei").and_then(Value::as_u64).unwrap_or(si);
            let start = ((si.saturating_sub(1)) as usize).min(buf.len());
            let end = (ei as usize).min(buf.len()).max(start); // ei is 1-based inclusive → end-exclusive
            let n = (end - start) as u64;
            if n == 0 {
                return;
            }
            buf.drain(start..end);
            events.push(EditEvent {
                at_ms,
                inserted_chars: 0,
                deleted_chars: n,
                keystrokes: n,
                at_offset: Some(si.saturating_sub(1)),
            });
        }
        Some("mlti") => {
            if let Some(mts) = op.get("mts").and_then(Value::as_array) {
                for sub in mts {
                    apply_op(sub, at_ms, buf, events);
                }
            }
        }
        _ => {} // style/setup ops carry no text
    }
}
