//! humanshipd native capture tool: watch an allow-listed editor via Accessibility,
//! build a metadata-only record, and mint a signed C2PA credential bound to the
//! document — then self-verify and write both the credential and the document so
//! the result can be checked independently.
//!
//! macOS-only for now (Accessibility). Run as a signed bundle (needs its own TCC
//! identity): `bash macos-capture/bundle.sh && open target/HumanshipdProbe.app`.
//! Output: `~/humanshipd-credential.c2pa`, `~/humanshipd-document.txt`, and a
//! summary at `/tmp/humanshipd-capture.log`.

fn main() {
    #[cfg(target_os = "macos")]
    macos::run();

    #[cfg(not(target_os = "macos"))]
    eprintln!(
        "humanshipd capture currently supports macOS (Accessibility). \
         Windows (UI Automation) and Linux (AT-SPI) adapters are planned."
    );
}

#[cfg(target_os = "macos")]
mod macos {
    use humanshipd_capture::ax;
    use humanshipd_capture::session::Capturer;
    use humanshipd_core::{build_record, credential};
    use std::fs::File;
    use std::io::Write;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    const CAPTURE_SECONDS: u64 = 30;
    const POLL_MS: u64 = 250;
    const ALLOW: &[&str] = &["TextEdit", "Microsoft Word"];
    const LOG_PATH: &str = "/tmp/humanshipd-capture.log";

    fn emit(log: &mut Option<File>, line: &str) {
        println!("{line}");
        if let Some(file) = log {
            let _ = writeln!(file, "{line}");
            let _ = file.flush();
        }
    }

    pub fn run() {
        let mut log = File::create(LOG_PATH).ok();

        if !ax::prompt_for_trust() {
            emit(
                &mut log,
                "Accessibility not granted — grant this app, then launch again.",
            );
            return;
        }

        emit(
            &mut log,
            &format!("Capturing {CAPTURE_SECONDS}s — type in {ALLOW:?} (keep it focused)…"),
        );

        let mut capturer = Capturer::new(
            ALLOW.iter().map(|s| s.to_string()).collect(),
            "native-1".to_string(),
        );
        let start = Instant::now();
        while start.elapsed().as_secs() < CAPTURE_SECONDS {
            capturer.tick(start.elapsed().as_millis() as u64);
            sleep(Duration::from_millis(POLL_MS));
        }

        let Some(input) = capturer.finish() else {
            emit(
                &mut log,
                "No writing captured. Did you type in TextEdit/Word while it was focused?",
            );
            return;
        };

        let document = input.final_text.clone();
        let record = build_record(&input);

        let manifest = match credential::issue_sidecar(&record, document.as_bytes()) {
            Ok(manifest) => manifest,
            Err(e) => {
                emit(&mut log, &format!("issue error: {e}"));
                return;
            }
        };

        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let credential_path = format!("{home}/humanshipd-credential.c2pa");
        let document_path = format!("{home}/humanshipd-document.txt");
        if std::fs::write(&credential_path, &manifest).is_err()
            || std::fs::write(&document_path, document.as_bytes()).is_err()
        {
            emit(&mut log, "write error");
            return;
        }

        match credential::read_sidecar(&manifest, document.as_bytes()) {
            Ok(readout) => {
                emit(&mut log, &format!("\nCredential : {credential_path}"));
                emit(&mut log, &format!("Document   : {document_path}"));
                emit(&mut log, &format!("surface    : {}", readout.record.surface.app));
                emit(&mut log, &format!("char_count : {}", readout.record.document_binding.char_count));
                emit(&mut log, &format!("ai_dump    : {}", readout.record.evidence_flags.large_unkeyed_insertions));
                emit(&mut log, &format!("valid      : {}", readout.valid));
                emit(&mut log, &format!("claim      : {}", readout.claim));
                emit(
                    &mut log,
                    &format!(
                        "\nVerify independently:\n  cargo run --example verify_credential -- \"{credential_path}\" \"{document_path}\""
                    ),
                );
            }
            Err(e) => emit(&mut log, &format!("verify error: {e}")),
        }
    }
}
