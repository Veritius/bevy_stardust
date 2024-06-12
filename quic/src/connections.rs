use bevy::prelude::*;
use quinn_proto::{Connection, ConnectionHandle};

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