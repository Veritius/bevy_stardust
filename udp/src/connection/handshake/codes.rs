#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub(super) enum HandshakeResponseCode {
    // The following values should not be changed.
    // They are used for good error responses in older versions.

    Continue = 0,

    // This code is returned when we can't recognise the code they send.
    // This should not be sent to a peer, that'd be weird.
    Unknown = u16::MAX,

    Unspecified = 1,
    MalformedPacket = 2,
    InvalidResponseCode = 3,

    IncompatibleTransportIdentifier = 4,
    IncompatibleTransportMajorVersion = 5,
    IncompatibleTransportMinorVersion = 6,

    // Anything below this point can be freely changed.

    IncompatibleApplicationIdentifier = 7,
    IncompatibleApplicationMajorVersion = 8,
    IncompatibleApplicationMinorVersion = 9,
}

impl HandshakeResponseCode {
    pub fn should_respond_on_rejection(&self) -> bool {
        match self {
            Self::MalformedPacket => false,
            _ => true,
        }
    }
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

            _ => Unknown,
        }
    }
}

impl std::fmt::Display for HandshakeResponseCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use HandshakeResponseCode::*;

        f.write_str(match self {
            Continue => "no error",
            Unknown => "unknown error code",

            MalformedPacket => "malformed packet",
            Unspecified => "no reason given",
            InvalidResponseCode => "response code was invalid",

            IncompatibleTransportIdentifier => "using incompatible transport",
            IncompatibleTransportMajorVersion => "incompatible transport major version",
            IncompatibleTransportMinorVersion => "incompatible transport minor version",

            IncompatibleApplicationIdentifier => "using different application",
            IncompatibleApplicationMajorVersion => "incompatible application major version",
            IncompatibleApplicationMinorVersion => "incompatible application minor version",
        })
    }
}

impl std::error::Error for HandshakeResponseCode {}