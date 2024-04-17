use bevy_ecs::prelude::*;

/// Used to intentionally reduce the performance of peers for testing purposes.
/// If applied to a `NetworkPeer` entity, reduces performance for that peer specifically.
#[derive(Debug, Component)]
pub struct NetworkPerformanceReduction {
    /// Chance to drop a packet when sending, if the transport is packet-based.
    /// This chance is from `0.0` (never) to `1.0` (always), with `0.5` dropping 50% of the time.
    pub packet_drop_chance: f32,

    /// Chance to mangle or otherwise invalidate a packet, if the transport is packet based.
    /// This chance is from `0.0` (never) to `1.0` (always), with `0.5` mangling 50% of the time.
    /// The degree to which the packet is mangled is up to the transport layer.
    pub packet_mangle_chance: f32,

    /// Artificial delay in transmitting, in milliseconds.
    /// 1000 milliseconds is the same as one second.
    pub transmit_delay_millis: u32,
}