use std::net::SocketAddr;
use bevy::prelude::*;
use crate::events::ConnectionEvents;

/// A QUIC connection.
#[derive(Component, Reflect)]
#[reflect(from_reflect = false, Component)]
pub struct Connection {
    #[reflect(ignore)]
    inner: Box<ConnectionInner>,
}

impl Connection {
    pub(crate) fn new(
        address: SocketAddr,
        quiche: quiche::Connection,
        events: ConnectionEvents,
    ) -> Self {
        Self {
            inner: Box::new(ConnectionInner {
                address,
                
                quiche,
                state: bevy_stardust_quic::Connection::new(),

                events,
            })
        }
    }
}

struct ConnectionInner {
    address: SocketAddr,

    quiche: quiche::Connection,
    state: bevy_stardust_quic::Connection,

    events: ConnectionEvents,
}