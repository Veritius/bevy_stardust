use std::fmt::Display;
use bytes::Bytes;
use super::codes::HandshakeResponseCode;

pub(super) enum HandshakeStateInner {
    ClientHello,
    ServerHello,
    Finished,
    Failed {
        reason: HandshakeFailureReason,
    },
}

#[derive(Debug)]
pub(super) enum HandshakeFailureReason {
    TimedOut,

    LocalRejection {
        code: HandshakeResponseCode,
        message: Option<Bytes>,
    },

    RemoteRejection {
        code: HandshakeResponseCode,
        message: Option<Bytes>,
    }
}

impl Display for HandshakeFailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use HandshakeFailureReason::*;

        match self {
            TimedOut => f.write_str("connection timed out"),

            LocalRejection { code, message } =>
                write!(f, "we rejected remote peer: {code} ({message:?})"),

            RemoteRejection { code, message } => 
                write!(f, "rejected by remote peer: {code} ({message:?})"),
        }
    }
}