//! Native Messaging host for humanshipd.
//!
//! A thin adapter: it frames stdio messages, manages a local key, and delegates
//! all credential logic to `humanshipd-core`. It holds no credential logic itself.

pub mod handler;
pub mod keystore;
pub mod messages;
pub mod protocol;
