//! Thin macOS Accessibility primitives shared by the capture tool and the probe.
//!
//! Note: the system-wide focused element returns -25204 even when trusted, so we
//! always go via `AXUIElementCreateApplication(pid)` for a specific app.
//!
//! `AXUIElementRef` is an opaque Core Foundation handle (not a Rust-dereferenced
//! pointer), so the pointer-arg lint doesn't apply to these wrappers.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use accessibility_sys::{
    kAXErrorSuccess, AXIsProcessTrustedWithOptions, AXUIElementCopyAttributeValue,
    AXUIElementCreateApplication, AXUIElementRef,
};
use core_foundation::base::{CFType, CFTypeRef, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use objc2_app_kit::NSWorkspace;
use std::ptr;

/// Check Accessibility trust, prompting with the system dialog if not granted.
pub fn prompt_for_trust() -> bool {
    let key = CFString::new("AXTrustedCheckOptionPrompt");
    let value = CFBoolean::true_value();
    let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
    unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) }
}

/// (pid, localized name) of the frontmost application.
pub fn frontmost_app() -> Option<(i32, String)> {
    let workspace = NSWorkspace::sharedWorkspace();
    let app = workspace.frontmostApplication()?;
    Some((app.processIdentifier(), localized_name(&app)))
}

/// (pid, name) of every running app whose localized name is in `targets`.
pub fn running_targets(targets: &[&str]) -> Vec<(i32, String)> {
    let workspace = NSWorkspace::sharedWorkspace();
    let apps = workspace.runningApplications();
    let mut out = Vec::new();
    for i in 0..apps.count() {
        let app = apps.objectAtIndex(i);
        let name = localized_name(&app);
        if targets.contains(&name.as_str()) {
            out.push((app.processIdentifier(), name));
        }
    }
    out
}

fn localized_name(app: &objc2_app_kit::NSRunningApplication) -> String {
    app.localizedName().map(|n| n.to_string()).unwrap_or_default()
}

/// Copy a string attribute, returning the AX error code on failure
/// (-98 null, -99 not a string).
pub fn copy_string(element: AXUIElementRef, attribute: &str) -> Result<String, i32> {
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

/// The focused UI element of the app with `pid`, via its per-application element.
pub fn focused_element(pid: i32) -> Result<AXUIElementRef, i32> {
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

pub fn role(element: AXUIElementRef) -> Option<String> {
    copy_string(element, "AXRole").ok()
}

pub fn value(element: AXUIElementRef) -> Option<String> {
    copy_string(element, "AXValue").ok()
}
