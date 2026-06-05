//! Diagnostic probe: list each running target app's focused-element role + text.
//! Run via the signed bundle; writes to /tmp/humanshipd-axprobe.log.

use humanshipd_capture::ax;
use std::fs::File;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

const POLL_SECONDS: u32 = 20;
const LOG_PATH: &str = "/tmp/humanshipd-axprobe.log";
const TARGETS: &[&str] = &["TextEdit", "Microsoft Word", "Word", "Scrivener", "Final Draft"];

fn emit(log: &mut Option<File>, line: &str) {
    println!("{line}");
    if let Some(file) = log {
        let _ = writeln!(file, "{line}");
        let _ = file.flush();
    }
}

fn preview(text: &str) -> String {
    text.chars()
        .map(|c| if c == '\n' { '⏎' } else { c })
        .take(60)
        .collect()
}

fn main() {
    let mut log = File::create(LOG_PATH).ok();
    emit(&mut log, &format!("AXIsProcessTrusted = {}", ax::prompt_for_trust()));
    emit(&mut log, &format!("\nPolling {POLL_SECONDS}s — type in TextEdit/Word:\n"));

    for tick in 0..POLL_SECONDS {
        let targets = ax::running_targets(TARGETS);
        if targets.is_empty() {
            emit(&mut log, &format!("[{tick:>2}s] no target apps running"));
        } else {
            for (pid, name) in targets {
                let line = match ax::focused_element(pid) {
                    Ok(element) => {
                        let role = ax::role(element).unwrap_or_else(|| "<no role>".to_string());
                        match ax::copy_string(element, "AXValue") {
                            Ok(text) => format!(
                                "[{tick:>2}s] {name} role={role} value={} | {}",
                                text.chars().count(),
                                preview(&text)
                            ),
                            Err(e) => format!("[{tick:>2}s] {name} role={role} AXValue err={e}"),
                        }
                    }
                    Err(e) => format!("[{tick:>2}s] {name} focused-element err={e}"),
                };
                emit(&mut log, &line);
            }
        }
        sleep(Duration::from_secs(1));
    }
    emit(&mut log, "\nDone.");
}
