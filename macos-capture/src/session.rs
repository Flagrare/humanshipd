//! Turns polled Accessibility values into a `core::SessionInput`.
//!
//! POC keystroke inference: a poll interval is short (~250ms), so a large jump in
//! text length in one interval can't be typing — it's a paste. We therefore mark
//! insertions ≥ the core's large-unkeyed threshold as keystroke-less (the AI-dump
//! signal) and smaller insertions as typed. A `CGEventTap` for true keystroke
//! correlation is a future refinement.

use crate::ax;
use humanshipd_core::{EditEvent, SessionInput, LARGE_UNKEYED_THRESHOLD};

pub struct Capturer {
    allow: Vec<String>,
    session_id: String,
    surface_app: String,
    prev_value: Option<String>,
    final_text: String,
    events: Vec<EditEvent>,
}

impl Capturer {
    pub fn new(allow: Vec<String>, session_id: String) -> Self {
        Self {
            allow,
            session_id,
            surface_app: String::new(),
            prev_value: None,
            final_text: String::new(),
            events: Vec::new(),
        }
    }

    /// Sample the focused text of the frontmost allow-listed app at `at_ms`.
    pub fn tick(&mut self, at_ms: u64) {
        let Some((pid, name)) = ax::frontmost_app() else {
            return;
        };
        if !self.allow.contains(&name) {
            return;
        }
        let Ok(element) = ax::focused_element(pid) else {
            return;
        };
        if ax::role(element).as_deref() != Some("AXTextArea") {
            return;
        }
        let Some(current) = ax::value(element) else {
            return;
        };
        if self.prev_value.as_ref() == Some(&current) {
            return;
        }

        let new_len = current.chars().count() as i64;
        let prev_len = self
            .prev_value
            .as_ref()
            .map(|p| p.chars().count() as i64)
            .unwrap_or(0);
        let delta = new_len - prev_len;

        let event = if delta > 0 {
            let inserted = delta as u64;
            let keystrokes = if inserted >= LARGE_UNKEYED_THRESHOLD {
                0
            } else {
                inserted
            };
            EditEvent {
                at_ms,
                inserted_chars: inserted,
                deleted_chars: 0,
                keystrokes,
            }
        } else if delta < 0 {
            EditEvent {
                at_ms,
                inserted_chars: 0,
                deleted_chars: (-delta) as u64,
                keystrokes: 0,
            }
        } else {
            // Same length, different content: a reformulation; treat as one keyed edit.
            EditEvent {
                at_ms,
                inserted_chars: 0,
                deleted_chars: 0,
                keystrokes: 1,
            }
        };

        self.events.push(event);
        self.surface_app = name;
        self.final_text = current.clone();
        self.prev_value = Some(current);
    }

    /// Build the session input, or `None` if nothing was captured.
    pub fn finish(self) -> Option<SessionInput> {
        if self.events.is_empty() {
            return None;
        }
        Some(SessionInput {
            session_id: self.session_id,
            surface_kind: "native-ax".to_string(),
            surface_app: self.surface_app,
            final_text: self.final_text,
            events: self.events,
        })
    }
}
