use bytes::Bytes;
use crate::varint::VarInt;

pub(super) struct ControlFrame {
    pub ident: ControlFrameIdent,
    pub payload: Bytes
}

pub(super) enum ControlFrameIdent {
    BeginClose,
    FullyClose,
}

impl TryFrom<VarInt> for ControlFrameIdent {
    type Error = ();

    fn try_from(value: VarInt) -> Result<Self, Self::Error> {
        use ControlFrameIdent::*;
        let c = u32::try_from(value)?;
        Ok(match c {
            0 => BeginClose,
            1 => FullyClose,
            _ => { return Err(()); }
        })
    }
}

impl From<ControlFrameIdent> for VarInt {
    fn from(value: ControlFrameIdent) -> Self {
        use ControlFrameIdent::*;
        let v = match value {
            BeginClose => 0,
            FullyClose => 1,
        };
        
        VarInt::from_u32(v)
    }
}