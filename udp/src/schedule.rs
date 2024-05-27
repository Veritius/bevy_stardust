use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub(crate) enum UdpSystemSet {
    /// Read packets
    PacketRead,

    /// Tick established connections.
    TickEstablished,

    /// Tick handshaking connections.
    TickHandshaking,

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