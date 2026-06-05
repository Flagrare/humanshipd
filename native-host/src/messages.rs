use serde::{Deserialize, Serialize};

/// Requests the extension's service worker sends to the host.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    /// Liveness/handshake check.
    Ping,
    /// Issue a credential from a captured session.
    Issue(IssueRequest),
    /// Any unrecognized request type.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct IssueRequest {
    pub session_id: String,
    pub surface_kind: String,
    pub surface_app: String,
    /// Used only to compute the document hash + char count; never stored in the badge.
    pub final_text: String,
    pub events: Vec<EventDto>,
}

#[derive(Debug, Deserialize)]
pub struct EventDto {
    pub at_ms: u64,
    pub inserted_chars: u64,
    pub deleted_chars: u64,
    pub keystrokes: u64,
}

/// Responses the host sends back to the extension.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    Pong { version: String },
    /// A C2PA credential (standalone `.c2pa` manifest store), base64-encoded.
    Credential { manifest_b64: String },
    Error { message: String },
}
