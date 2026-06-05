//! Opt-in credential registry: maps a content fingerprint (an ISCC code) to the
//! credential issued for that content.
//!
//! The point of the registry is recovery: if a document's sidecar credential is
//! lost or stripped, a verifier can recompute the document's ISCC fingerprint
//! locally and ask the registry for the matching credential. Crucially, the
//! registry only ever stores **fingerprints and credentials** — never the
//! document text. Publishing to it is the author's explicit, opt-in choice.

use std::collections::HashMap;
use std::sync::Mutex;

/// An in-memory fingerprint → credential store (thread-safe).
#[derive(Default)]
pub struct Registry {
    by_fingerprint: Mutex<HashMap<String, Vec<u8>>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a credential under its content fingerprint (ISCC code).
    pub fn register(&self, fingerprint: &str, credential: Vec<u8>) {
        self.by_fingerprint
            .lock()
            .expect("registry lock")
            .insert(fingerprint.to_string(), credential);
    }

    /// Look up a credential by content fingerprint (exact match for now;
    /// near-match by ISCC similarity is a future enhancement).
    pub fn lookup(&self, fingerprint: &str) -> Option<Vec<u8>> {
        self.by_fingerprint
            .lock()
            .expect("registry lock")
            .get(fingerprint)
            .cloned()
    }

    pub fn len(&self) -> usize {
        self.by_fingerprint.lock().expect("registry lock").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_then_lookup_round_trips() {
        let registry = Registry::new();
        registry.register("ISCC:EAA-example", b"credential-bytes".to_vec());

        assert_eq!(
            registry.lookup("ISCC:EAA-example"),
            Some(b"credential-bytes".to_vec())
        );
        assert_eq!(registry.lookup("ISCC:not-registered"), None);
        assert_eq!(registry.len(), 1);
    }
}
