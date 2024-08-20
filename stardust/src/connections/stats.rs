use std::{ops::{Deref, DerefMut}, time::Duration};
use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;

/// Round-trip time estimate for [peer entities].
/// 
/// Round-trip time (RTT) is the duration of time that it takes
/// for one message to be sent to a peer, and then a response
/// to be sent back by the recipient. This estimate is set by
/// the transport layer managing a connection.
/// 
/// [peer entities]: crate::connections
#[derive(Debug, Default, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
pub struct PeerRtt(pub Duration);

impl Deref for PeerRtt {
    type Target = Duration;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PeerRtt {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
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