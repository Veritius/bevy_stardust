use std::net::UdpSocket;
use bevy::prelude::*;

/// A QUIC endpoint, corresponding to a single UDP socket.
/// 
/// All [connections](crate::Connection) 'belong' to an Endpoint, which they use for I/O.
#[derive(Component)]
pub struct Endpoint {
    socket: UdpSocket,
}