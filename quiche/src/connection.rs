use bevy::prelude::*;

/// A QUIC connection.
#[derive(Component, Reflect)]
#[reflect(from_reflect = false, Component)]
pub struct Connection {
    #[reflect(ignore)]
    inner: ConnectionInner,
}

pub(crate) struct ConnectionInner {

}

pub(crate) struct ConnectionState {
    quiche: quiche::Connection,
    state: bevy_stardust_quic::Connection,
}