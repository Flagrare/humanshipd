use crate::messages::{EventDto, IssueRequest, Request, Response};
use base64::Engine;
use humanshipd_core::{build_record, credential, EditEvent, SessionInput};

/// Dispatch a request to a response. Never panics; errors become `Response::Error`.
pub fn process(request: Request) -> Response {
    match request {
        Request::Ping => Response::Pong {
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        Request::Issue(req) => issue(req),
        Request::Unknown => Response::Error {
            message: "unknown request type".to_string(),
        },
    }
}

fn issue(req: IssueRequest) -> Response {
    let final_text = req.final_text.clone();
    let input = SessionInput {
        session_id: req.session_id,
        surface_kind: req.surface_kind,
        surface_app: req.surface_app,
        final_text: req.final_text,
        events: req.events.into_iter().map(to_event).collect(),
    };

    let record = build_record(&input);
    match credential::issue_sidecar(&record, final_text.as_bytes()) {
        Ok(manifest) => Response::Credential {
            manifest_b64: base64::engine::general_purpose::STANDARD.encode(manifest),
        },
        Err(e) => Response::Error {
            message: e.to_string(),
        },
    }
}

fn to_event(dto: EventDto) -> EditEvent {
    EditEvent {
        at_ms: dto.at_ms,
        inserted_chars: dto.inserted_chars,
        deleted_chars: dto.deleted_chars,
        keystrokes: dto.keystrokes,
        at_offset: dto.at_offset,
    }
}
