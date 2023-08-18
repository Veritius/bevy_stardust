use std::collections::BTreeMap;
use bevy::prelude::*;
use crate::{server::prelude::*, shared::{channels::{id::ChannelId, incoming::IncomingNetworkMessages}, payload::Payload}, client::peers::Server};
use super::RemoteServerUdpSocket;

const PACKET_HEADER_SIZE: usize = 3;

pub(super) fn receive_packets_system(
    remote: Option<Res<RemoteServerUdpSocket>>,
    channel_registry: Res<ChannelRegistry>,
    // channels: Query<(&ChannelData, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    mut server: Query<&mut IncomingNetworkMessages, With<Server>>,
) {
    let Some(remote) = remote else { return };
    
    // Read all packets from the server
    // Unlike the server, this is single-threaded.
    let mut buffer = [0u8; 1500];
    let mut inter_map = BTreeMap::new();
    loop {
        // Check if we've run out of packets
        let Ok(octets) = remote.0.recv(&mut buffer) else { break };

        // Discard packet, too small to be useful.
        if octets <= 3 { continue; }

        // Get channel ID and check it exists
        let channel_id = ChannelId::from(TryInto::<[u8;3]>::try_into(&buffer[..3]).unwrap());
        if !channel_registry.channel_exists(channel_id) { continue; } // Channel doesn't exist

        // Clone message data
        let mut payload = Vec::with_capacity(octets-1);
        for oct in &buffer[PACKET_HEADER_SIZE..=octets] { payload.push(*oct); }

        // Add to map
        let entry = inter_map.entry(channel_id).or_insert(Vec::with_capacity(1));
        entry.push(Payload::new(0, 0, payload));
    }
    
    // Write to IncomingNetworkMessages
    let mut server_messages = server.single_mut();
    loop {
        let Some((id, payloads)) = inter_map.pop_first() else { break };
        for payload in payloads {
            server_messages.append(id, payload);
        }
    }
}