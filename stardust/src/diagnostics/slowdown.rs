use std::time::Duration;
use bevy::prelude::*;

/// Instructs transport layers to drop packets randomly, simulating an unstable connection.
/// 
/// This value ranges between `0.0` (never drop) to `1.0` (always drop), with `0.5` dropping 50% of the time.
#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Debug, Default, Component)]
pub struct DropPackets(#[reflect(@0.0..=1.0)] pub f32);

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