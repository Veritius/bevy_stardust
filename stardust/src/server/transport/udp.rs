//! Native UDP transport layer for servers.

use bevy::prelude::*;
use crate::shared::{
    scheduling::{TransportReadPackets, TransportSendPackets},
    protocol::Protocol
};

pub struct ServerUdpTransportPlugin;
impl Plugin for ServerUdpTransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(TransportReadPackets, receive_packets_system);
        app.add_systems(TransportSendPackets, send_packets_system);
    }
}

fn receive_packets_system(
    protocol: Res<Protocol>,
) {

}

fn send_packets_system(
    protocol: Res<Protocol>,
) {

}