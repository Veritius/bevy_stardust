use bevy::prelude::*;
use bevy_stardust_quic::Connection as ConnectionState;
use quinn::{Connecting, Connection};

/// Represents one Quinn connection.
#[derive(Component)]
pub struct QuinnConnection {
    connection: ConnectionInner,
    qs_state: Box<ConnectionState>,
}

impl QuinnConnection {
    pub(crate) fn connecting(connecting: Connecting) -> Self {
        Self {
            connection: ConnectionInner::Connecting(connecting),
            qs_state: Box::new(ConnectionState::new()),
        }
    }
}

enum ConnectionInner {
    Connecting(Connecting),
    Established(Connection),
}