use crate::messages::{EventDto, IssueRequest, Request, Response};
use humanshipd_core::{
    anchor_badge, build_record, sign_record, EditEvent, KeyPair, LocalTsa, SessionInput,
};

/// Per-request context: the client signing key and the time-anchoring authority.
/// A fresh `tsa` (with the current time) is constructed for each request by the caller.
pub struct Ctx<'a> {
    pub client_key: &'a KeyPair,
    pub tsa: LocalTsa,
}

/// Dispatch a request to a response. Never panics; errors become `Response::Error`.
pub fn process(request: Request, ctx: &Ctx) -> Response {
    match request {
        Request::Ping => Response::Pong {
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        Request::Issue(req) => issue(req, ctx),
        Request::Unknown => Response::Error {
            message: "unknown request type".to_string(),
        },
    }
}

fn issue(req: IssueRequest, ctx: &Ctx) -> Response {
    let input = SessionInput {
        session_id: req.session_id,
        surface_kind: req.surface_kind,
        surface_app: req.surface_app,
        final_text: req.final_text,
        events: req.events.into_iter().map(to_event).collect(),
    };

    let record = build_record(&input);
    match sign_record(record, ctx.client_key).and_then(|badge| anchor_badge(badge, &ctx.tsa)) {
        Ok(badge) => Response::Badge {
            badge: Box::new(badge),
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
    }
}
