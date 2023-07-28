use bevy::{prelude::*, ecs::system::SystemParam};

/// If the client is currently connected to a remote server.
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, States)]
pub enum RemoteConnectionStatus {
    /// No connection exists or is being attempted.
    #[default]
    Unconnected,
    /// The remote server has not yet responded to any messages.
    Connecting,
    /// The remote server is processing something before it accepts the user.
    Authenticating,
    /// The client is fully connected with the remote server.
    Connected,
}

#[derive(SystemParam)]
pub struct ConnectionManager<'w> {
    status: Res<'w, State<RemoteConnectionStatus>>,
}

/// The reason the server disconnected the client. This information is given voluntarily by the server.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ConnectionRejectionReason {
    /// The server didn't give a reason.
    #[default]
    NoneGiven,

    /// The server's protocol ID and the client's protocol ID were different.
    BadProtocol,

    /// The server's authentication system rejected the client.
    AuthDenied,

    /// The server was at the limit for connected clients.
    PlayerCap,

    /// The server returned a string.
    Custom(String),
}