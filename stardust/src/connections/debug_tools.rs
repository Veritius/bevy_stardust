//! Debugging tools that may be unwanted in release builds.

use std::time::{Duration, Instant};
use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;

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
/// [`PeerRtt`]: crate::connections::PeerRtt
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

/// Instructs transport layers to drop packets randomly, simulating an unstable connection.
/// 
/// This value ranges between `0.0` (never drop) to `1.0` (always drop), with `0.5` dropping 50% of the time.
#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Debug, Default, Component)]
pub struct DropPackets(#[reflect(@0.0..=1.0)] pub f32);

impl DropPackets {
    /// Never drop packets.
    pub const NEVER: Self = Self(0.0);

    /// Always drop packets.
    pub const ALWAYS: Self = Self(1.0);
}

/// Instructs transport layers to artifically increase latency, simulating a distant connection.
/// 
/// This latency increase is implemented by the transport layer, as a minimum latency value.
/// You can think of it as a function `min(a,b)` where `a` is their real latency, and `b` is the value in this component.
#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Debug, Default, Component)]
pub struct SimulateLatency(pub Duration);

impl From<Duration> for SimulateLatency {
    #[inline]
    fn from(value: Duration) -> Self {
        Self(value)
    }
}