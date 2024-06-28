use std::time::Instant;
use bevy::prelude::*;

/// A statistics tracking component for [peer entities].
/// 
/// When added to a peer, it tracks data about the peer's connection.
/// This is useful for debugging and system administrator tools.
/// These values are set by the transport layer managing the connection.
/// 
/// Note that round-trip time is not tracked in this component.
/// RTT is tracked in its own component, called [`PeerRtt`].
/// 
/// [peer entities]: crate::connections
#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Default, Component)]
#[non_exhaustive]
pub struct PeerStats {
    /// The last time any data was received by the transport layer.
    /// May be `None` if data has never been received.
    pub last_recv: Option<Instant>,

    /// Outgoing data in kilobits per second, including overhead from the transport layer.
    pub all_kbps_out: u32,

    /// Outgoing data in kilobits per second, only counting bytes in individual messages.
    /// If messages are sent piecemeal (in multiple chunks received on different ticks),
    /// the received data is still counted.
    pub msg_kbps_out: u32,

    /// Incoming data in kilobits per second, including overhead from the transport layer.
    pub all_kbps_in: u32,

    /// Incoming data in kilobits per second, only counting bytes in individual messages.
    /// If messages are sent piecemeal (in multiple chunks received on different ticks),
    /// the received data is still counted.
    pub msg_kbps_in: u32,
}