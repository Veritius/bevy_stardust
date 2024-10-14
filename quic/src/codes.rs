/// Different codes that can be returned when disconnecting.
/// 
/// Intended for translation into varints when consumed by other crates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DisconnectCode {
    ProtocolError,
}

impl From<DisconnectCode> for u32 {
    fn from(value: DisconnectCode) -> Self {
        match value {
            DisconnectCode::ProtocolError => 0,
        }
    }
}

impl TryFrom<u32> for DisconnectCode {
    type Error = UnknownCode;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        return Ok(match value {
            0 => DisconnectCode::ProtocolError,

            _ => return Err(UnknownCode),
        });
    }
}

/// Different codes that can be returned when closing a stream.
/// 
/// Intended for translation into varints when consumed by other crates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResetCode {

}

impl From<ResetCode> for u32 {
    fn from(value: ResetCode) -> Self {
        match value {
        }
    }
}

impl TryFrom<u32> for ResetCode {
    type Error = UnknownCode;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        return Ok(match value {
            _ => return Err(UnknownCode),
        });
    }
}

/// Error value for TryFrom impls for codes.
pub struct UnknownCode;