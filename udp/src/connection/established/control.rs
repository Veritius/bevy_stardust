use crate::varint::VarInt;

pub(super) enum ControlFrameIdent {
    BeginClose,
}

impl TryFrom<VarInt> for ControlFrameIdent {
    type Error = ();

    fn try_from(value: VarInt) -> Result<Self, Self::Error> {
        let c = u32::try_from(value)?;
        Ok(match c {
            0 => Self::BeginClose,
            _ => { return Err(()); }
        })
    }
}

impl From<ControlFrameIdent> for VarInt {
    fn from(value: ControlFrameIdent) -> Self {
        let v = match value {
            ControlFrameIdent::BeginClose => 0,
        };
        
        VarInt::from_u32(v)
    }
}