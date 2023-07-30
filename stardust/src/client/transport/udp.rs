//! Native UDP transport layer for clients.

mod packets;
mod attempt;

use bevy::prelude::{Plugin, App};
use crate::shared::scheduling::{TransportReadPackets, TransportSendPackets};
use self::packets::{receive_packets_system, send_packets_system};

/// A simple transport layer over native UDP sockets.
pub struct ClientUdpTransportPlugin;
impl Plugin for ClientUdpTransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(TransportReadPackets, receive_packets_system);
        app.add_systems(TransportSendPackets, send_packets_system);
    }
}