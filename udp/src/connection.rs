use std::net::SocketAddr;
use bevy_ecs::prelude::*;

/// A UDP connection.
#[derive(Component)]
#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect), reflect(from_reflect = false))]
pub struct Connection {
    #[cfg_attr(feature="reflect", reflect(ignore))]
    pub(crate) local_address: SocketAddr,
}