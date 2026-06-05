//! Native macOS Accessibility capture adapter for humanshipd.
//!
//! `ax` wraps the Accessibility primitives; `session` turns polled values into a
//! `core::SessionInput`. Both are macOS-only; the binaries fall back to a clear
//! message on other platforms (Windows/Linux adapters are planned).

#[cfg(target_os = "macos")]
pub mod ax;
#[cfg(target_os = "macos")]
pub mod session;
