use bevy::prelude::*;
use crate::{shared::{prelude::*, channels::outgoing::OutgoingOctetStringsAccessor, reliability::PeerSequenceData}, client::peers::Server};
use super::RemoteServerUdpSocket;

pub(super) fn send_packets_system(
    remote: Option<Res<RemoteServerUdpSocket>>,
    outgoing: OutgoingOctetStringsAccessor,
    registry: Res<ChannelRegistry>,
    channels: Query<(&ChannelData, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    mut server: Query<&mut PeerSequenceData, With<Server>>,
) { 
    let Some(remote) = remote else { return };
    let mut server_seq = server.single_mut();

    // Reliability information
    let mut reliable_amount: u16 = 0;
    for channel in outgoing.by_channel() {
        if channels.get(registry.get_from_id(channel.id()).unwrap()).unwrap().2.is_none() { continue; }
        for (_, _) in channel.read() {
            reliable_amount += 1;
        }
    }

    let highest_sequence_id = server_seq.local_sequence.wrapping_add(reliable_amount.into());

    for outgoing in outgoing.by_channel() {
        // Get channel data
        let id = outgoing.id();
        let (channel_data, ordered, reliable, fragmented) =
            channels.get(registry.get_from_id(id).unwrap()).unwrap();
        let (ordered, reliable, fragmented) =
            (ordered.is_some(), reliable.is_some(), fragmented.is_some());

        for (target, octets) in outgoing.strings().read() {
            // Panics if incorrect sendtargets are used.
            // Largely redundant.
            match target {
                SendTarget::Single(_) => {},
                SendTarget::Multiple(_) => unimplemented!(),
                SendTarget::Broadcast => unimplemented!(),
            }

            // TODO: Figure out a better way to do this
            let mut payload = Vec::with_capacity(3 + octets.len());

            // Always present information
            for b in id.bytes() { payload.push(b); }
            for b in highest_sequence_id.bytes() { payload.push(b); }

            // Packet global seq id
            if reliable {
                for b in server_seq.next().bytes() { payload.push(b); }
            }

            // The payload itself
            for b in octets.as_slice() { payload.push(*b); }

            // Send data
            remote.0.send(&payload).unwrap();
        }
    }

    // Sanity check
    assert_eq!(highest_sequence_id, server_seq.local_sequence);
}