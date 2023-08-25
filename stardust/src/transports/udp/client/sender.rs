use bevy::prelude::*;
use crate::{channels::{registry::ChannelRegistry, outgoing::{OutgoingOctetStringsAccessor, SendTarget}, config::*}, client::peers::Server};
use super::RemoteServerUdpSocket;

pub(super) fn send_packets_system(
    remote: Option<Res<RemoteServerUdpSocket>>,
    outgoing: OutgoingOctetStringsAccessor,
    registry: Res<ChannelRegistry>,
    channels: Query<(&ChannelData, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    mut server: Query<(), With<Server>>,
) { 
    let Some(remote) = remote else { return };
    let mut server_seq = server.single_mut();

    for outgoing in outgoing.by_channel() {
        // Get channel data
        let id = outgoing.id();
        let (_channel_data, ordered, reliable, fragmented) =
            channels.get(registry.get_from_id(id).unwrap()).unwrap();
        let (_ordered, reliable, _fragmented) =
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
            let cid = (id.0 + 1.into())
                .expect("Too many channels! The transport layer imposes a restriction of 2^24-1.");
            for b in cid.bytes() { payload.push(b); }

            // The payload itself
            for b in octets.as_slice() { payload.push(*b); }

            // Send data
            remote.0.send(&payload).unwrap();
        }
    }

    // Sanity check
    // assert_eq!(highest_sequence_id, server_seq.local_sequence);
}