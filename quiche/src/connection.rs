use std::net::SocketAddr;
use bevy::prelude::*;

/// A QUIC connection.
#[derive(Component, Reflect)]
#[reflect(from_reflect = false, Component)]
pub struct Connection {
    #[reflect(ignore)]
    inner: ConnectionInner,
}

struct ConnectionInner {

}

pub(crate) struct ConnectionState {
    address: SocketAddr,

    quiche: quiche::Connection,
    state: bevy_stardust_quic::Connection,
}