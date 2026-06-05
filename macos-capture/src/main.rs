//! Probe: read the focused UI element's text via the macOS Accessibility API.
//!
//! This is the de-risking first step of the native capture adapter. It proves we
//! can read live text from an arbitrary focused app (TextEdit, then Word) before
//! we build the full diff + keystroke-correlation pipeline.
//!
//! Run: `cargo run -p humanshipd-macos-capture`
//! Requires granting this binary "Accessibility" permission in
//! System Settings → Privacy & Security → Accessibility.

use accessibility_sys::{
    kAXErrorSuccess, AXIsProcessTrusted, AXUIElementCopyAttributeValue,
    AXUIElementCreateSystemWide, AXUIElementRef,
};
use core_foundation::base::{CFTypeRef, TCFType};
use core_foundation::string::{CFString, CFStringRef};
use std::ptr;
use std::thread::sleep;
use std::time::Duration;

/// Seconds to poll so the operator can switch focus to a target app and type.
const POLL_SECONDS: u32 = 20;

/// Read a string attribute (e.g. "AXValue") from an AX element, if present.
fn copy_string_attribute(element: AXUIElementRef, attribute: &str) -> Option<String> {
    let attr = CFString::new(attribute);
    let mut value: CFTypeRef = ptr::null();
    let err = unsafe {
        AXUIElementCopyAttributeValue(element, attr.as_concrete_TypeRef(), &mut value)
    };
    if err != kAXErrorSuccess || value.is_null() {
        return None;
    }
    // Interpret the returned value as a CFString and own it.
    let cf = unsafe { CFString::wrap_under_create_rule(value as CFStringRef) };
    Some(cf.to_string())
}

/// Read the AX element currently focused system-wide, if any.
fn copy_focused_element() -> Option<AXUIElementRef> {
    let system_wide = unsafe { AXUIElementCreateSystemWide() };
    let attr = CFString::new("AXFocusedUIElement");
    let mut value: CFTypeRef = ptr::null();
    let err = unsafe {
        AXUIElementCopyAttributeValue(system_wide, attr.as_concrete_TypeRef(), &mut value)
    };
    if err != kAXErrorSuccess || value.is_null() {
        return None;
    }
    Some(value as AXUIElementRef)
}

fn focused_text() -> Option<String> {
    let focused = copy_focused_element()?;
    copy_string_attribute(focused, "AXValue")
}

/// A single-line, length-capped preview of captured text for the console.
fn preview(text: &str) -> String {
    let one_line: String = text.chars().map(|c| if c == '\n' { '⏎' } else { c }).collect();
    let capped: String = one_line.chars().take(60).collect();
    if one_line.chars().count() > 60 {
        format!("{capped}…")
    } else {
        capped
    }
}

fn main() {
    if !unsafe { AXIsProcessTrusted() } {
        eprintln!(
            "Accessibility permission NOT granted.\n\
             Grant it in System Settings → Privacy & Security → Accessibility\n\
             (add your terminal app, or target/debug/humanshipd-macos-capture),\n\
             then re-run."
        );
        return;
    }

    println!("Polling the focused text field for {POLL_SECONDS}s.");
    println!("Click into TextEdit (then Word) and type — captured text appears below:\n");

    let mut last: Option<String> = None;
    for tick in 0..POLL_SECONDS {
        let current = focused_text();
        if current != last {
            match &current {
                Some(text) => println!("[{tick:>2}s] {:>4} chars | {}", text.chars().count(), preview(text)),
                None => println!("[{tick:>2}s] (no focused text element)"),
            }
            last = current;
        }
        sleep(Duration::from_secs(1));
    }
    println!("\nDone.");
}
