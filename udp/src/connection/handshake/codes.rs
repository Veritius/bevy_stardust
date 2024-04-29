use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum HandshakeResponseCode {
    // The following values should not be changed.
    // They are used for good error responses in older versions.

    Continue = 0,

    // This code is returned when we can't recognise the code they send.
    // This should not be sent to a peer.
    Unknown = u16::MAX,

    Unspecified = 1,
    MalformedResponse = 2,
    InvalidResponse = 3,

    IncompatibleTransportIdentifier = 4,
    IncompatibleTransportMajorVersion = 5,
    IncompatibleTransportMinorVersion = 6,

    // Anything below this point can be freely changed.

    IncompatibleApplicationIdentifier = 7,
    IncompatibleApplicationMajorVersion = 8,
    IncompatibleApplicationMinorVersion = 9,

    ServerNotListening = 10,
    ApplicationCloseEvent = 11,
    UnacceptableBehavior = 12,
}

impl From<u16> for HandshakeResponseCode {
    fn from(value: u16) -> Self {
        use HandshakeResponseCode::*;

        match value {
            0 => Continue,

            1 => Unspecified,
            2 => MalformedResponse,
            3 => InvalidResponse,

            4 => IncompatibleTransportIdentifier,
            5 => IncompatibleTransportMajorVersion,
            6 => IncompatibleTransportMinorVersion,

            7 => IncompatibleApplicationIdentifier,
            8 => IncompatibleApplicationMajorVersion,
            9 => IncompatibleApplicationMinorVersion,

            10 => ServerNotListening,
            11 => ApplicationCloseEvent,
            12 => UnacceptableBehavior,

            _ => Unknown,
        }
    }
}

impl Display for HandshakeResponseCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use HandshakeResponseCode::*;

        f.write_str(match self {
            Continue => "no error",
            Unknown => "unknown",

            Unspecified => "unspecified",
            MalformedResponse => "malformed packet",
            InvalidResponse => "invalid response",

            IncompatibleTransportIdentifier => "incompatible transport",
            IncompatibleTransportMajorVersion => "incompatible transport major version",
            IncompatibleTransportMinorVersion => "incompatible transport minor version",

            IncompatibleApplicationIdentifier => "incompatible application",
            IncompatibleApplicationMajorVersion => "incompatible application major version",
            IncompatibleApplicationMinorVersion => "incompatible application minor version",

            ServerNotListening => "server not accepting connections",
            ApplicationCloseEvent => "closed by application",
            UnacceptableBehavior => "behaved strangely too many times",
        })
    }
}