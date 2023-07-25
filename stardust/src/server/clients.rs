use std::net::UdpSocket;
use bevy::prelude::*;

/// Represents a connected client as an entity.
/// 
/// Despawning the entity or otherwise removing the component will silently disconnect the client.
#[derive(Debug, Component)]
pub struct Client {
    #[cfg(not(feature="expose_internals"))]
    pub(super) socket: UdpSocket,
    #[cfg(feature="expose_internals")]
    pub socket: UdpSocket,
}