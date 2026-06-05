//! Native macOS Accessibility capture adapter for humanshipd.
//!
//! `ax` wraps the Accessibility primitives; `session` turns polled values into a
//! `core::SessionInput`. The binaries (`main` capture tool, `probe` diagnostic)
//! and `humanshipd-core` do the rest.

pub mod ax;
pub mod session;
