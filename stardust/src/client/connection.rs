//! Connection status information.

use bevy::prelude::*;

/// If the client is currently connected to a remote server.
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, States)]
pub enum RemoteConnectionStatus {
    /// No connection exists or is being attempted.
    #[default]
    Unconnected,
    /// The remote server has not yet responded to any messages.
    Connecting,
    /// The client is fully connected with the remote server.
    Connected,
}

impl RemoteConnectionStatus {
    /// Returns true if a connection is established.
    pub fn connected(&self) -> bool {
        *self == Self::Connected
    }
}