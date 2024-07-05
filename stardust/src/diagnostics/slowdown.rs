use std::time::Duration;
use bevy::prelude::*;

/// Reduces the performance of I/O entities and types that it is attached to.
/// This merely instructs transport layers as to what they should do,
/// and how they handle these values is defined per transport layer.
#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Debug, Component)]
pub struct NetworkPerformanceReduction {
    /// Chance to drop a packet when sending, if the transport is packet-based.
    /// This chance is from `0.0` (never) to `1.0` (always), with `0.5` dropping 50% of the time.
    #[reflect(@0.0..=1.0)]
    pub packet_drop_chance: f32,

    /// Controls the **minimum** RTT that will be used, based on the transport layer's estimates.
    /// If the peer would have a lesser RTT, an artificial delay is added to increase it.
    /// This is good for simulating connections with extremely high latency.
    pub simulate_rtt: Duration,
}

impl Default for NetworkPerformanceReduction {
    fn default() -> Self {
        Self {
            packet_drop_chance: 0.0,
            simulate_rtt: Duration::ZERO,
        }
    }
}