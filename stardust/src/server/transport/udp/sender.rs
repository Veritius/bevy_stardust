use std::{sync::Mutex, net::{UdpSocket, SocketAddr}};
use bevy::{prelude::*, tasks::TaskPool};
use once_cell::sync::Lazy;
use crate::{shared::{channels::{outgoing::OutgoingOctetStringsAccessor, id::ChannelId}, octetstring::OctetString}, server::clients::Client};
use super::{UdpClient, ports::PortBindings};

// TODO
// Despite the parallelism, this is pretty inefficient.
// It iterates over things when it doesn't need to several times.

pub(super) fn send_packets_system(
    // registry: Res<ChannelRegistry>,
    // channels: Query<(&ChannelData, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    mut ports: ResMut<PortBindings>,
    clients: Query<&UdpClient, With<Client>>,
    outgoing: OutgoingOctetStringsAccessor,
) {
    // Create task pool
    let pool = TaskPool::new();

    // List of clients to remove in case of a mistake
    let port_removals = Lazy::new(|| Mutex::new(Vec::with_capacity(1)));

    // Intentional borrow to prevent moves
    let clients = &clients;
    let outgoing = &outgoing;
    let port_removals = &port_removals;

    // Create tasks for all ports
    pool.scope(|s| {
        for port in ports.iter() {
            s.spawn(async move {
                let (_, socket, cls) = port;
                for cl in cls {
                    // Get client entity
                    let Ok(udp_comp) = clients.get(*cl) else {
                        error!("Entity {:?} was in PortBindings but was not a client", *cl);
                        port_removals.lock().unwrap().push(*cl);
                        continue;
                    };

                    // Iterate all channels
                    let channels = outgoing.by_channel();
                    for channel in channels {
                        let channel_id = channel.id();
                        // Iterate all octet strings
                        for (target, octets) in channel.strings().read() {
                            // Check this message is for this client
                            if target.excludes(*cl) { continue }
                            // Send packet
                            send_udp_packet(socket, channel_id, &udp_comp.address, octets)
                        }
                    }
                }
            });
        }
    });

    // Remove clients if there's an issue
    let lock = port_removals.lock().unwrap();
    for v in lock.iter() {
        ports.remove_client(*v);
    }
}

fn send_udp_packet(
    socket: &UdpSocket,
    channel: ChannelId,
    address: &SocketAddr,
    octets: &OctetString,
) {
    let mut udp_payload = Vec::with_capacity(1500);
    for octet in channel.as_bytes() { udp_payload.push(octet); }
    for octet in octets.as_slice() { udp_payload.push(*octet); }
    socket.send_to(&udp_payload, address).unwrap();
}