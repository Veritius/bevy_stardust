use std::time::Instant;
use bevy::prelude::*;
use bytes::Bytes;
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

impl QuicConnection {
    /// Begins closing the connection.
    pub fn close(&mut self, reason: Bytes) {
        self.inner.close(
            Instant::now(),
            DisconnectCode::AppDisconnect.try_into().unwrap(),
            reason
        );
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DisconnectCode {
    Invalid,

    Unspecified,
    AppDisconnect,
    NotListening,
}

impl From<VarInt> for DisconnectCode {
    fn from(value: VarInt) -> Self {
        use DisconnectCode::*;
        match u64::from(value) {
            0 => Unspecified,
            1 => AppDisconnect,
            2 => NotListening,

            _ => Invalid,
        }
    }
}

impl TryFrom<DisconnectCode> for VarInt {
    type Error = ();

    fn try_from(value: DisconnectCode) -> Result<Self, Self::Error> {
        use DisconnectCode::*;
        return Ok(VarInt::from_u32(match value {
            // Special case: this variant can't be sent
            Invalid => { return Err(()) },

            Unspecified => 0,
            AppDisconnect => 1,
            NotListening => 2,
        }));
    }
}