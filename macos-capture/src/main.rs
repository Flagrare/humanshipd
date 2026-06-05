//! Diagnostic probe for the macOS Accessibility capture path.
//!
//! Prints trust status, the focused element's role, and exact AX error codes each
//! second, and *prompts* for Accessibility permission if it is missing. Used to
//! pin down why focused-text capture returns nothing before building the real
//! adapter.
//!
//! Run: `cargo run -p humanshipd-macos-capture`
//! Then click into TextEdit (and Word) and type while it polls.

use accessibility_sys::{
    kAXErrorSuccess, AXIsProcessTrustedWithOptions, AXUIElementCopyAttributeValue,
    AXUIElementCreateSystemWide, AXUIElementRef,
};
use core_foundation::base::{CFType, CFTypeRef, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use std::fs::File;
use std::io::Write;
use std::ptr;
use std::thread::sleep;
use std::time::Duration;

const POLL_SECONDS: u32 = 20;
/// When launched as a `.app` (no console), output also goes here.
const LOG_PATH: &str = "/tmp/humanshipd-axprobe.log";

/// Write a line to stdout and (best-effort) to the log file.
fn emit(log: &mut Option<File>, line: &str) {
    println!("{line}");
    if let Some(file) = log {
        let _ = writeln!(file, "{line}");
        let _ = file.flush();
    }
}

/// Copy a string attribute, returning the AX error code on failure
/// (or -99 if the value exists but is not a string).
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

/// Copy the system-wide focused UI element, returning the AX error on failure.
fn copy_focused_element() -> Result<AXUIElementRef, i32> {
    let system_wide = unsafe { AXUIElementCreateSystemWide() };
    let attr = CFString::new("AXFocusedUIElement");
    let mut value: CFTypeRef = ptr::null();
    let err = unsafe {
        AXUIElementCopyAttributeValue(system_wide, attr.as_concrete_TypeRef(), &mut value)
    };
    if err != kAXErrorSuccess {
        return Err(err);
    }
    if value.is_null() {
        return Err(-98);
    }
    Ok(value as AXUIElementRef)
}

fn preview(text: &str) -> String {
    let one_line: String = text
        .chars()
        .map(|c| if c == '\n' { '⏎' } else { c })
        .take(60)
        .collect();
    one_line
}

/// Prompt for Accessibility permission (shows the system dialog if not granted).
fn prompt_for_trust() -> bool {
    let key = CFString::new("AXTrustedCheckOptionPrompt");
    let value = CFBoolean::true_value();
    let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
    unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) }
}

fn main() {
    let mut log = File::create(LOG_PATH).ok();

    let trusted = prompt_for_trust();
    emit(&mut log, &format!("AXIsProcessTrusted = {trusted}"));
    if !trusted {
        emit(
            &mut log,
            "→ A permission dialog should have appeared. Grant Accessibility to this\n\
             app, then launch it again.",
        );
    }
    emit(
        &mut log,
        &format!("\nPolling {POLL_SECONDS}s — click into TextEdit, then Word, and type:\n"),
    );

    for tick in 0..POLL_SECONDS {
        let line = match copy_focused_element() {
            Ok(element) => {
                let role = copy_string(element, "AXRole").unwrap_or_else(|e| format!("<err {e}>"));
                match copy_string(element, "AXValue") {
                    Ok(text) => format!(
                        "[{tick:>2}s] role={role} value={} chars | {}",
                        text.chars().count(),
                        preview(&text)
                    ),
                    Err(value_err) => {
                        let selected = copy_string(element, "AXSelectedText").ok();
                        format!("[{tick:>2}s] role={role} AXValue err={value_err} selected={selected:?}")
                    }
                }
            }
            Err(e) => format!("[{tick:>2}s] no focused element (AXError {e})"),
        };
        emit(&mut log, &line);
        sleep(Duration::from_secs(1));
    }
    emit(&mut log, &format!("\nDone. (log written to {LOG_PATH})"));
}
