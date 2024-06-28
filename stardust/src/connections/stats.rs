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