#[derive(Debug, Clone, Copy)]
pub(crate) enum ResetCode {
    Unspecified,
    Violation,
}

impl From<u32> for ResetCode {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Unspecified,
            1 => Self::Violation,

            _ => Self::Unspecified,
        }
    }
}

impl From<ResetCode> for u32 {
    fn from(value: ResetCode) -> Self {
        match value {
            ResetCode::Unspecified => 0,
            ResetCode::Violation => 1,
        }
    }
}

impl From<quinn_proto::VarInt> for ResetCode {
    fn from(value: quinn_proto::VarInt) -> Self {
        let v = value.into_inner();
        if v > 2u64.pow(32) { return Self::Unspecified };
        return Self::from(v as u32);
    }
}

impl From<ResetCode> for quinn_proto::VarInt {
    fn from(value: ResetCode) -> Self {
        quinn_proto::VarInt::from_u32(value.into())
    }
}

impl std::fmt::Display for ResetCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ResetCode::Unspecified => "unspecified or unknown",
            ResetCode::Violation => "unspecified violation",
        })
    }
}