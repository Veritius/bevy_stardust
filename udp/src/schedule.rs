use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub(crate) enum PreUpdateSet {
    /// Read packets
    PacketRead,

    /// Tick established connections.
    TickEstablished,

    /// Handle unknown, potential new connections.
    HandleUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub(crate) enum UpdateSet {
    /// Tick handshaking connections.
    TickHandshaking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub(crate) enum PostUpdateSet {
    /// Established connections pack frames.
    FramePacking,

    /// Handshake component sends information.
    HandshakeSend,

    /// Send packets
    PacketSend,

    /// Handle closing connections.
    CloseConnections,

    /// Handle closing endpoints.
    CloseEndpoints,

    /// Update statistics and diagnostic values.
    UpdateStatistics,
}