//! Native UDP transport layer for clients.

mod manager;
mod receiver;
mod sender;
mod attempt;

pub use manager::UdpClientManager;

use std::net::UdpSocket;
use bevy::prelude::{Plugin, App, Resource, OnTransition};
use crate::{scheduling::*, prelude::NetworkMode};
use self::{receiver::receive_packets_system, sender::send_packets_system, attempt::connection_attempt_system};

pub(super) fn setup_udp_client(app: &mut App) {
    app.add_systems(OnTransition {
        from: NetworkMode::Offline,
        to: NetworkMode::Client,
    }, initialise::initialise_udp_client_system);

    app.add_systems(OnTransition {
        from: NetworkMode::Client,
        to: NetworkMode::Offline,
    }, shutdown::shutdown_udp_client_system);
}

/// A simple transport layer over native UDP sockets.
pub struct ClientUdpTransportPlugin;
impl Plugin for ClientUdpTransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(TransportReadPackets, connection_attempt_system);
        app.add_systems(TransportReadPackets, receive_packets_system);
        app.add_systems(TransportSendPackets, send_packets_system);
    }
}

#[derive(Resource)]
struct RemoteServerUdpSocket(pub UdpSocket);