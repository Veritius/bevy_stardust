use bevy_ecs::prelude::*;
use crate::backend::connection::{ConnectionCreation, ConnectionHandle};

#[derive(Component)]
pub struct Connection {
    inner: ConnectionInner,
}

enum ConnectionInner {
    Trying(Box<ConnectionCreation>),
    Established(Box<ConnectionHandle>),
}