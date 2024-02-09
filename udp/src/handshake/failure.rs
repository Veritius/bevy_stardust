use std::fmt::Display;

pub enum HandshakeFailureMessage {
    UnsupportedTransportVersion,
    MandatoryFlagMismatch,
}

impl Display for HandshakeFailureMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedTransportVersion => f.write_str("Transport version is not supported"),
            Self::MandatoryFlagMismatch => f.write_str("A required flag was not present"),
        }
    }
}