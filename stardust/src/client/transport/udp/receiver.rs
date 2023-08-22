use bevy::prelude::*;
use crate::{server::prelude::*, shared::{channels::{id::ChannelId, incoming::IncomingNetworkMessages}, payload::Payload, reliability::{SequenceId, PeerSequenceData}}, client::peers::Server};
use super::RemoteServerUdpSocket;

const PACKET_HEADER_SIZE: usize = 5;

pub(super) fn receive_packets_system(
    remote: Option<Res<RemoteServerUdpSocket>>,
    channel_registry: Res<ChannelRegistry>,
    channels: Query<(&ChannelData, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    mut server: Query<(&mut PeerSequenceData, &mut IncomingNetworkMessages), With<Server>>,
) {
    let Some(remote) = remote else { return };
    let (mut server_seq, mut server_messages) = server.single_mut();
    
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
        let (channel_data, ordered, reliable, fragmented) =
            channels.get(channel_ent).unwrap();
        let (ordered, reliable, fragmented) =
            (ordered.is_some(), reliable.is_some(), fragmented.is_some());

        let mut cutoff = PACKET_HEADER_SIZE;

        // Read highest sequence ID value
        server_seq.try_highest([buffer[3], buffer[4]].into());

        // Reliability stuff
        if reliable {
            if octets_read < 7 { continue; } // Reliable message without reliability data
            let sequence: SequenceId = [buffer[5], buffer[6]].into();
            server_seq.mark_received(sequence);
            cutoff += 2;
        }

        // Clone message data
        let mut payload = Vec::with_capacity(octets_read);
        for oct in &buffer[cutoff..=octets_read] { payload.push(*oct); }
        let payload = Payload::new(0, 0, payload);

        // Add to map
        pending.push((channel_id, payload));
    }

    // Get packets that were missed
    let missed = server_seq.complete_cycle().collect::<Vec<_>>();
    info!("Missed packets from server: {:?}", &missed);

    // Write to IncomingNetworkMessages
    loop {
        let Some((id, payload)) = pending.pop() else { break };
        server_messages.append(id, payload);
    }
}