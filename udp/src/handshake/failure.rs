use std::fmt::Display;

pub enum HandshakeFailureMessage {
    /// All available encryption methods are refused or encryption support is disabled via the feature flag.
    EncryptionNotSupported,
}

impl Display for HandshakeFailureMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandshakeFailureMessage::EncryptionNotSupported => f.write_str("No encryption methods are supported"),
        }
    }
}