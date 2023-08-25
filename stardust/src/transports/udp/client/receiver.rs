use bevy::prelude::*;
use crate::{channels::{registry::ChannelRegistry, config::*, incoming::IncomingNetworkMessages, id::ChannelId}, client::peers::Server, octets::payload::Payload};
use super::RemoteServerUdpSocket;

const PACKET_HEADER_SIZE: usize = 5;

pub(super) fn receive_packets_system(
    remote: Option<Res<RemoteServerUdpSocket>>,
    channel_registry: Res<ChannelRegistry>,
    channels: Query<(&ChannelData, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    mut server: Query<&mut IncomingNetworkMessages, With<Server>>,
) {
    let Some(remote) = remote else { return };
    let mut server_messages = server.single_mut();
    
    // Read all packets from the server
    // Unlike the server, this is single-threaded.
    let mut buffer = [0u8; 1500];
    let mut pending = vec![];
    loop {
        // Check if we've run out of packets
        let Ok(octets_read) = remote.0.recv(&mut buffer) else { break };

        // Discard packet, too small to be useful.
        if octets_read <= 6 { continue; }

        // Check channel ID and get config
        let channel_id = ChannelId::from(TryInto::<[u8;3]>::try_into(&buffer[..3]).unwrap());
        if !channel_registry.channel_exists(channel_id) { continue; } // Channel doesn't exist
        let channel_ent = channel_registry.get_from_id(channel_id).unwrap();
        let (_channel_data, ordered, reliable, fragmented) =
            channels.get(channel_ent).unwrap();
        let (_ordered, reliable, _fragmented) =
            (ordered.is_some(), reliable.is_some(), fragmented.is_some());

        let cutoff = PACKET_HEADER_SIZE;

        // Clone message data
        let mut payload = Vec::with_capacity(octets_read);
        for oct in &buffer[cutoff..=octets_read] { payload.push(*oct); }
        let payload = Payload::new(0, 0, payload);

        // Add to map
        pending.push((channel_id, payload));
    }

    // Write to IncomingNetworkMessages
    loop {
        let Some((id, payload)) = pending.pop() else { break };
        server_messages.append(id, payload);
    }
}