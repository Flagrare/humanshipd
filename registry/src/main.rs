//! Minimal HTTP server for the opt-in credential registry.
//!
//! Endpoints (it only ever sees fingerprints + credentials, never content):
//!   POST /register        { "fingerprint": "<iscc>", "credential_b64": "<base64>" }
//!   GET  /lookup/{iscc}    → the credential bytes, or 404
//!
//! Run: `cargo run -p humanshipd-registry` (binds 127.0.0.1:8787).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use base64::Engine;
use humanshipd_registry::Registry;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
struct RegisterRequest {
    fingerprint: String,
    credential_b64: String,
}

#[tokio::main]
async fn main() {
    let registry = Arc::new(Registry::new());
    let app = Router::new()
        .route("/register", post(register))
        .route("/lookup/{fingerprint}", get(lookup))
        .with_state(registry);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8787")
        .await
        .expect("bind");
    println!("humanshipd registry on http://127.0.0.1:8787 — stores fingerprints + credentials only");
    axum::serve(listener, app).await.expect("serve");
}

async fn register(
    State(registry): State<Arc<Registry>>,
    Json(request): Json<RegisterRequest>,
) -> StatusCode {
    match base64::engine::general_purpose::STANDARD.decode(request.credential_b64) {
        Ok(credential) => {
            registry.register(&request.fingerprint, credential);
            StatusCode::CREATED
        }
        Err(_) => StatusCode::BAD_REQUEST,
    }
}

async fn lookup(
    State(registry): State<Arc<Registry>>,
    Path(fingerprint): Path<String>,
) -> Result<Vec<u8>, StatusCode> {
    registry.lookup(&fingerprint).ok_or(StatusCode::NOT_FOUND)
}
