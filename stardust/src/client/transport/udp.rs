//! Native UDP transport layer for clients.

use bevy::prelude::*;
use crate::shared::scheduling::{TransportReadPackets, TransportSendPackets};

/// A simple transport layer over native UDP sockets.
pub struct ClientUdpTransportPlugin;
impl Plugin for ClientUdpTransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(TransportReadPackets, receive_packets_system);
        app.add_systems(TransportSendPackets, send_packets_system);
    }
}

fn receive_packets_system(
    
) {

}

fn send_packets_system(
    
) {

}