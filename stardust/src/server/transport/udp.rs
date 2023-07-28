//! Native UDP transport layer for servers.

use std::net::UdpSocket;

use bevy::{prelude::*, tasks::TaskPool};
use crate::{shared::{scheduling::{TransportReadPackets, TransportSendPackets}, channels::{components::{OrderedChannel, ReliableChannel, FragmentedChannel, ChannelData}, id::ChannelId, registry::ChannelRegistry}}, server::clients::Client};

/// A simple transport layer over native UDP sockets.
pub struct ServerUdpTransportPlugin;
impl Plugin for ServerUdpTransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(TransportReadPackets, receive_packets_system);
        app.add_systems(TransportSendPackets, send_packets_system);
    }
}

/// A client connected with the `ServerUdpTransportPlugin` transport layer.
#[derive(Component)]
pub struct UdpClient(UdpSocket);

/// Maximum packet length that can be sent/received before fragmentation.
const MAX_PACKET_LENGTH: usize = 1500;
/// The amount of bytes that will always be present in all packages.
const PACKET_HEADER_SIZE: usize = 3;

fn receive_packets_system(
    clients: Query<(Entity, &Client, &UdpClient)>,
    channels: Query<(&ChannelData, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    channel_registry: Res<ChannelRegistry>,
) {
    // Create thread pool for processing
    let pool = TaskPool::new();

    // Explicitly borrow to prevent moves
    let channels = &channels;
    let channel_registry = &channel_registry;

    // Receive packets from connected clients
    let mut client_packets = pool.scope(|s| {
        for (client_id, _, client_udp) in clients.iter() {
            let client_id = client_id.clone();
            s.spawn(async move {
                let mut packets = vec![];
                let mut buffer = [0u8; MAX_PACKET_LENGTH];

                // Read all packets
                loop {
                    if let Ok(octets) = client_udp.0.recv(&mut buffer) {
                        // Discard packet, too small to be useful.
                        if octets <= 3 { continue; }

                        // Get channel ID and check it exists
                        let channel_id = ChannelId::try_from(&buffer[0..=3]).unwrap();
                        if !channel_registry.channel_exists(channel_id) { break; }

                        // Copy octets from buffer
                        let idx = octets - PACKET_HEADER_SIZE - 1;
                        let mut packet = Vec::with_capacity(idx);
                        for i in (PACKET_HEADER_SIZE + 1)..idx {
                            packet.push(buffer[i]);
                        }

                        // Insert packet data
                        packets.push((client_id, channel_id, packet.into_boxed_slice()));
                    } else {
                        // We're done reading packets
                        break;
                    }
                }

                // Return packets
                packets
            });
        }
    });
}

fn send_packets_system(
    
) {

}