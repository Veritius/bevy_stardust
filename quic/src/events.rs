use std::net::SocketAddr;
use bevy::prelude::*;
use crate::Credentials;

/// Send to try and connect to a remote peer with `endpoint`.
#[derive(Event)]
pub struct TryConnectEvent {
    /// The endpoint that manages the I/O for the connection.
    pub endpoint: Entity,

    /// The address of the remote connection,
    pub address: SocketAddr,

    /// The name of the server, used for certificate authentication.
    pub server_name: Box<str>,

    /// Credentials for authenticating this peer during the handshake.
    pub credentials: Option<Credentials>,
}