use std::fmt;

/// Errors surfaced by the credential core.
#[derive(Debug)]
pub enum CoreError {
    /// Canonicalization / (de)serialization failure.
    Serialization(String),
    /// Hex/byte-length decoding failure.
    Encoding(String),
    /// Cryptographic key/signature failure.
    Crypto(String),
    /// Document text-extraction failure (unreadable or unsupported format).
    Format(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::Serialization(msg) => write!(f, "serialization error: {msg}"),
            CoreError::Encoding(msg) => write!(f, "encoding error: {msg}"),
            CoreError::Crypto(msg) => write!(f, "crypto error: {msg}"),
            CoreError::Format(msg) => write!(f, "format error: {msg}"),
        }
    }
}

impl std::error::Error for CoreError {}
