pub(super) fn response_code_from_int(value: u16) -> HandshakeCode {
    if value == 0 { return HandshakeCode::Ok; }
    HandshakeCode::Err(HandshakeErrorCode::from(value))
}

#[derive(Debug)]
pub(super) enum HandshakeCode {
    Ok,
    Err(HandshakeErrorCode),
}

impl From<HandshakeOkCode> for HandshakeCode {
    fn from(_: HandshakeOkCode) -> Self {
        Self::Ok
    }
}

impl From<HandshakeErrorCode> for HandshakeCode {
    fn from(value: HandshakeErrorCode) -> Self {
        Self::Err(value)
    }
}

#[derive(Debug)]
pub(super) struct HandshakeOkCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub(super) enum HandshakeErrorCode {
    // Do not add anything equal to zero!
    // Zero is used for the 'all okay' value.

    // The following values should not be changed.
    // They are used for good error responses in older versions.

    // This code is returned when we can't recognise the code they send.
    // You should not send this to a remote peer as an actual error code.
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

impl From<u16> for HandshakeErrorCode {
    fn from(value: u16) -> Self {
        use HandshakeErrorCode::*;

        match value {
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

impl std::fmt::Display for HandshakeErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use HandshakeErrorCode::*;

        f.write_str(match self {
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

impl std::error::Error for HandshakeErrorCode {}