use bevy::prelude::*;
use bevy_stardust::{scheduling::{NetworkRead, NetworkWrite}, channels::registry::ChannelRegistry};

/// A transport layer for Stardust that uses native UDP sockets.
pub struct UdpTransportPlugin;

impl Plugin for UdpTransportPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}