use bevy::prelude::*;
use quinn_proto::{Connection, ConnectionHandle, VarInt};

/// A QUIC connection, attached to an endpoint.
/// 
/// # Safety
/// This component must always stay in the same [`World`] as it was created in.
/// Being put into another `World` will lead to undefined behavior.
#[derive(Component)]
pub struct QuicConnection {
    pub(crate) handle: ConnectionHandle,
    pub(crate) inner: Box<Connection>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DisconnectCode {
    Invalid,

    Unspecified,
}

impl From<VarInt> for DisconnectCode {
    fn from(value: VarInt) -> Self {
        use DisconnectCode::*;
        match u64::from(value) {
            0 => Unspecified,
            _ => Invalid,
        }
    }
}

impl TryFrom<DisconnectCode> for VarInt {
    type Error = ();

    fn try_from(value: DisconnectCode) -> Result<Self, Self::Error> {
        return Ok(VarInt::from_u32(match value {
            // Special case: this variant can't be sent
            DisconnectCode::Invalid => { return Err(()) },

            DisconnectCode::Unspecified => 0,
        }));
    }
}