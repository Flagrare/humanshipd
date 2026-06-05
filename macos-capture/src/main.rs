//! Diagnostic probe for the macOS Accessibility capture path.
//!
//! Reads the focused text of specific TARGET apps (TextEdit, Word, …) by name —
//! regardless of which app is frontmost — so capture can be observed from another
//! window. Uses `AXUIElementCreateApplication(pid)` (the system-wide focused
//! element returns -25204 even when trusted).
//!
//! Build the signed bundle and run it (a bare `cargo run` binary lacks a TCC
//! identity): `bash macos-capture/bundle.sh && open target/HumanshipdProbe.app`.

use accessibility_sys::{
    kAXErrorSuccess, AXIsProcessTrustedWithOptions, AXUIElementCopyAttributeValue,
    AXUIElementCreateApplication, AXUIElementRef,
};
use core_foundation::base::{CFType, CFTypeRef, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use objc2_app_kit::NSWorkspace;
use std::fs::File;
use std::io::Write;
use std::ptr;
use std::thread::sleep;
use std::time::Duration;

const POLL_SECONDS: u32 = 20;
const LOG_PATH: &str = "/tmp/humanshipd-axprobe.log";
/// Apps we try to read by name, regardless of frontmost status.
const TARGET_APPS: &[&str] = &["TextEdit", "Microsoft Word", "Word", "Scrivener", "Final Draft"];

fn emit(log: &mut Option<File>, line: &str) {
    println!("{line}");
    if let Some(file) = log {
        let _ = writeln!(file, "{line}");
        let _ = file.flush();
    }
}

/// (pid, name) of every running app whose name is in TARGET_APPS.
fn running_targets() -> Vec<(i32, String)> {
    let workspace = NSWorkspace::sharedWorkspace();
    let apps = workspace.runningApplications();
    let mut out = Vec::new();
    for i in 0..apps.count() {
        let app = apps.objectAtIndex(i);
        let name = app.localizedName().map(|n| n.to_string()).unwrap_or_default();
        if TARGET_APPS.iter().any(|t| name == *t) {
            out.push((app.processIdentifier(), name));
        }
    }
    out
}

fn copy_string(element: AXUIElementRef, attribute: &str) -> Result<String, i32> {
    let attr = CFString::new(attribute);
    let mut value: CFTypeRef = ptr::null();
    let err =
        unsafe { AXUIElementCopyAttributeValue(element, attr.as_concrete_TypeRef(), &mut value) };
    if err != kAXErrorSuccess {
        return Err(err);
    }
    if value.is_null() {
        return Err(-98);
    }
    let cf = unsafe { CFType::wrap_under_create_rule(value) };
    cf.downcast::<CFString>().map(|s| s.to_string()).ok_or(-99)
}

fn focused_element_for_pid(pid: i32) -> Result<AXUIElementRef, i32> {
    let app = unsafe { AXUIElementCreateApplication(pid) };
    if app.is_null() {
        return Err(-97);
    }
    let attr = CFString::new("AXFocusedUIElement");
    let mut value: CFTypeRef = ptr::null();
    let err = unsafe { AXUIElementCopyAttributeValue(app, attr.as_concrete_TypeRef(), &mut value) };
    if err != kAXErrorSuccess {
        return Err(err);
    }
    if value.is_null() {
        return Err(-98);
    }
    Ok(value as AXUIElementRef)
}

fn preview(text: &str) -> String {
    text.chars()
        .map(|c| if c == '\n' { '⏎' } else { c })
        .take(60)
        .collect()
}

fn prompt_for_trust() -> bool {
    let key = CFString::new("AXTrustedCheckOptionPrompt");
    let value = CFBoolean::true_value();
    let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
    unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) }
}

fn report_target(tick: u32, pid: i32, name: &str) -> String {
    match focused_element_for_pid(pid) {
        Ok(element) => {
            let role = copy_string(element, "AXRole").unwrap_or_else(|e| format!("<err {e}>"));
            match copy_string(element, "AXValue") {
                Ok(text) => format!(
                    "[{tick:>2}s] {name} role={role} value={} chars | {}",
                    text.chars().count(),
                    preview(&text)
                ),
                Err(value_err) => {
                    let selected = copy_string(element, "AXSelectedText").ok();
                    format!("[{tick:>2}s] {name} role={role} AXValue err={value_err} selected={selected:?}")
                }
            }
        }
        Err(e) => format!("[{tick:>2}s] {name} (pid {pid}) focused-element err={e}"),
    }
}

fn main() {
    let mut log = File::create(LOG_PATH).ok();

    let trusted = prompt_for_trust();
    emit(&mut log, &format!("AXIsProcessTrusted = {trusted}"));
    emit(
        &mut log,
        &format!("\nPolling {POLL_SECONDS}s — type in TextEdit / Word (no need to keep them frontmost):\n"),
    );

    for tick in 0..POLL_SECONDS {
        let targets = running_targets();
        if targets.is_empty() {
            emit(&mut log, &format!("[{tick:>2}s] no target apps running (open TextEdit or Word)"));
        } else {
            for (pid, name) in targets {
                let line = report_target(tick, pid, &name);
                emit(&mut log, &line);
            }
        }
        sleep(Duration::from_secs(1));
    }
    emit(&mut log, &format!("\nDone. (log: {LOG_PATH})"));
}
