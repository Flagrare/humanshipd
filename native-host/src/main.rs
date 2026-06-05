use humanshipd_core::{KeyPair, LocalTsa};
use humanshipd_host::handler::{process, Ctx};
use humanshipd_host::messages::{Request, Response};
use humanshipd_host::{keystore, protocol};
use std::io;

/// Honest POC authority label — surfaced in tokens so verifiers never assume a real TSA.
const AUTHORITY: &str = "humanshipd:local-poc";

fn main() -> io::Result<()> {
    let client_key = KeyPair::from_seed(&keystore::load_or_create_seed("client.key")?);
    let tsa_seed = keystore::load_or_create_seed("tsa.key")?;

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let stdout = io::stdout();
    let mut writer = stdout.lock();

    while let Some(payload) = protocol::read_message(&mut reader)? {
        let gen_time = chrono::Utc::now().to_rfc3339();
        let ctx = Ctx {
            client_key: &client_key,
            tsa: LocalTsa::new(&tsa_seed, AUTHORITY, gen_time),
        };

        let response = match serde_json::from_slice::<Request>(&payload) {
            Ok(request) => process(request, &ctx),
            Err(e) => Response::Error {
                message: format!("invalid request: {e}"),
            },
        };

        let bytes = serde_json::to_vec(&response).unwrap_or_else(|e| {
            format!("{{\"type\":\"error\",\"message\":\"serialize failed: {e}\"}}").into_bytes()
        });
        protocol::write_message(&mut writer, &bytes)?;
    }

    Ok(())
}
