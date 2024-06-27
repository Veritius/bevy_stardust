use std::time::Duration;
use bevy::prelude::*;

/// Round-trip time estimate for [peer entities].
/// 
/// Round-trip time (RTT) is the duration of time that it takes
/// for one message to be sent to a peer, and then a response
/// to be sent back by the recipient. This estimate is set by
/// the transport layer managing a connection.
/// 
/// [peer entities]: crate::connections
#[derive(Debug, Default, Clone, Copy, Component)]
pub struct PeerRtt(pub Duration);

impl AsRef<Duration> for PeerRtt {
    #[inline]
    fn as_ref(&self) -> &Duration {
        &self.0
    }
}

impl AsMut<Duration> for PeerRtt {
    #[inline]
    fn as_mut(&mut self) -> &mut Duration {
        &mut self.0
    }
}

impl From<PeerRtt> for Duration {
    #[inline]
    fn from(value: PeerRtt) -> Self {
        value.0
    }
}

impl From<Duration> for PeerRtt {
    #[inline]
    fn from(value: Duration) -> Self {
        Self(value)
    }
}

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
#[reflect(Default)]
#[non_exhaustive]
pub struct PeerStats {
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