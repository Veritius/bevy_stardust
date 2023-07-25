use std::net::{UdpSocket, SocketAddr};
use bevy::prelude::*;

/// Represents a connected client as an entity.
/// 
/// Despawning the entity or otherwise removing the component will silently disconnect the client.
#[derive(Debug, Component)]
pub struct Client {
    pub(super) socket: UdpSocket,
    pub(super) address: SocketAddr,
}