#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub(crate) enum HandshakeResponseCode {
    Continue = 0,

    Unspecified = 1,
    MalformedPacket = 2,
    InvalidResponseCode = 3,

    IncompatibleTransportIdentifier = 4,
    IncompatibleTransportMajorVersion = 5,
    IncompatibleTransportMinorVersion = 6,

    IncompatibleApplicationIdentifier = 7,
    IncompatibleApplicationMajorVersion = 8,
    IncompatibleApplicationMinorVersion = 9,

    ServerNotListening = 10,
    ApplicationCloseEvent = 11,

    // This code is returned when we can't recognise the code they send.
    // This should not be sent to a peer, that'd be weird.
    Unknown = u16::MAX,
}

impl From<u16> for HandshakeResponseCode {
    fn from(value: u16) -> Self {
        use HandshakeResponseCode::*;

        match value {
            0 => Continue,

            1 => Unspecified,
            2 => MalformedPacket,
            3 => InvalidResponseCode,

            4 => IncompatibleTransportIdentifier,
            5 => IncompatibleTransportMajorVersion,
            6 => IncompatibleTransportMinorVersion,

            7 => IncompatibleApplicationIdentifier,
            8 => IncompatibleApplicationMajorVersion,
            9 => IncompatibleApplicationMinorVersion,

            10 => ServerNotListening,
            11 => ApplicationCloseEvent,

            _ => Unknown,
        }
    }
}

impl std::fmt::Display for HandshakeResponseCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use HandshakeResponseCode::*;

        f.write_str(match self {
            Continue => "no error",

            MalformedPacket => "malformed packet",
            Unspecified => "no reason given",
            InvalidResponseCode => "response code was invalid",

            IncompatibleTransportIdentifier => "using incompatible transport",
            IncompatibleTransportMajorVersion => "incompatible transport major version",
            IncompatibleTransportMinorVersion => "incompatible transport minor version",

            IncompatibleApplicationIdentifier => "using different application",
            IncompatibleApplicationMajorVersion => "incompatible application major version",
            IncompatibleApplicationMinorVersion => "incompatible application minor version",

            ServerNotListening => "server not accepting connections",
            ApplicationCloseEvent => "close event sent during handshake",

            Unknown => "unknown error code",
        })
    }
}