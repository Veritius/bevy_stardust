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
        let channel_id = ChannelId::from_bytes(&buffer[0..=3].try_into().unwrap());
        if !channel_registry.channel_exists(channel_id) { break; } // Channel doesn't exist

        // Clone message data
        let mut payload = Vec::with_capacity(octets-1);
        payload.clone_from_slice(&buffer[PACKET_HEADER_SIZE..octets]);

        // Add to map
        let entry = inter_map.entry(channel_id).or_insert(Vec::with_capacity(1));
        entry.push(Payload::new(0, 0, payload));
    }
    
    // Change Vec<Payload> into Payloads in map
    let mut final_map = BTreeMap::new();
    loop {
        let popped = inter_map.pop_first();
        let Some((id, payload)) = popped else { break };
        final_map.insert(id, payload.into());
    }

    // Set new map and return
    let mut server_ent = server.single_mut();
    server_ent.0 = final_map;
}