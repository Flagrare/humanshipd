use humanshipd_host::handler::process;
use humanshipd_host::messages::{Request, Response};
use humanshipd_host::protocol;
use std::io;

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let stdout = io::stdout();
    let mut writer = stdout.lock();

    while let Some(payload) = protocol::read_message(&mut reader)? {
        let response = match serde_json::from_slice::<Request>(&payload) {
            Ok(request) => process(request),
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
