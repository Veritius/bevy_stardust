use bevy::{prelude::*, ecs::system::SystemParam};

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