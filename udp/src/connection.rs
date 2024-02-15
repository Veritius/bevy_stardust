use std::net::SocketAddr;
use bevy_ecs::prelude::*;

/// A UDP connection.
#[derive(Component)]
pub struct Connection(pub(crate) ConnectionInner);

pub(crate) struct ConnectionInner {
    pub address: SocketAddr,
}