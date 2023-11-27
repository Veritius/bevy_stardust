use bevy::prelude::*;
use bevy_stardust::scheduling::{TransportReadPackets, TransportSendPackets};
use crate::{receiving::receive_packets_system, sending::send_packets_system};

/// A transport layer for Stardust that uses native UDP sockets.
pub struct UdpTransportPlugin;

impl Plugin for UdpTransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(TransportReadPackets, receive_packets_system);
        app.add_systems(TransportSendPackets, send_packets_system);
    }
}