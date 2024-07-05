use std::fmt::Display;

#[derive(Debug)]
pub(super) enum ResetCode {
    Unspecified,
}

impl From<u32> for ResetCode {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Unspecified,

            _ => Self::Unspecified,
        }
    }
}

impl From<ResetCode> for u32 {
    fn from(value: ResetCode) -> Self {
        match value {
            ResetCode::Unspecified => 0,
        }
    }
}

impl From<ResetCode> for quinn_proto::VarInt {
    fn from(value: ResetCode) -> Self {
        quinn_proto::VarInt::from_u32(value.into())
    }
}

impl Display for ResetCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ResetCode::Unspecified => "unspecified or unknown",
        })
    }
}