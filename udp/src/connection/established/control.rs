use crate::varint::VarInt;
use super::frames::frames::RecvFrame;

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

pub(super) fn handle_control_frame(
    frame: RecvFrame,
) -> Result<(), u16> {
    let ident = frame.ident.ok_or(1024u16)?;

    use ControlFrameIdent::*;
    match ControlFrameIdent::try_from(ident) {
        Ok(BeginClose) => {
            todo!()
        },

        Err(_) => { return Err(1024u16); },
    }
}