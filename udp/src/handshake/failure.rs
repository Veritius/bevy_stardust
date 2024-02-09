use std::fmt::Display;

pub enum HandshakeFailureMessage {
    UnsupportedTransportVersion,
    EncryptionNotSupported,
    MandatoryFlagMismatch,
}

impl Display for HandshakeFailureMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedTransportVersion => f.write_str("Transport version is not supported"),
            Self::EncryptionNotSupported => f.write_str("No encryption methods are supported"),
            Self::MandatoryFlagMismatch => f.write_str("A required flag was not present"),
        }
    }
}