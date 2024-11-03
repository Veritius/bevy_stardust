use bevy_ecs::prelude::*;
use crate::backend::connection::ConnectionRef;

/// A QUIC connection.
#[derive(Component)]
pub struct Connection {
    inner: ConnectionRef,
}