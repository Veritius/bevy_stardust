use bevy::prelude::*;
use crate::{shared::channels::outgoing::OutgoingOctetStringsAccessor, server::{clients::Client, prelude::*}};
use super::UdpClient;

pub(super) fn send_packets_system(
    // registry: Res<ChannelRegistry>,
    // channels: Query<(&ChannelData, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    outgoing: OutgoingOctetStringsAccessor,
    clients: Query<&UdpClient, With<Client>>,
) {
    // TODO: Parallelism?

    // Iterate all channels
    for idx in outgoing.all() {
        // Get channel data
        let channel = idx.id();
        let data = idx.data();

        // Nothing to send
        if data.count() == 0 { continue }

        // Iterate all messages
        let octets = data.read();
        for (target, octets) in octets {
            // Assemble payload
            let mut payload = Vec::with_capacity(3 + octets.len());
            for b in channel.as_bytes() { payload.push(b); }
            for b in octets.as_slice() { payload.push(*b); }

            // Send to target
            match target {
                SendTarget::Single(target) => {
                    // TODO: Deal with sending packets to clients that don't exist
                    let Ok(client) = clients.get(*target) else { continue };
                    client.socket.send(&payload).unwrap();
                },
                SendTarget::Multiple(targets) => {
                    for target in targets.iter() {
                        let Ok(client) = clients.get(*target) else { continue };
                        client.socket.send(&payload).unwrap();
                    }
                },
                SendTarget::Broadcast => {
                    for client in clients.iter() {
                        client.socket.send(&payload).unwrap();
                    }
                },
            }
        }
    }
}