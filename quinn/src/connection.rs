use bevy::prelude::*;
use quinn::{Connecting, Connection};

/// Represents one Quinn connection.
#[derive(Component)]
pub struct QuinnConnection {
    connection: ConnectionInner,
}

impl QuinnConnection {
    pub(crate) fn connecting(connecting: Connecting) -> Self {
        Self {
            connection: ConnectionInner::Connecting(connecting),
        }
    }
}

enum ConnectionInner {
    Connecting(Connecting),
    Established(Connection),
}