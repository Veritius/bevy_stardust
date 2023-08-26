use std::{sync::{Mutex, MutexGuard}, net::UdpSocket, collections::BTreeMap};
use bevy::{prelude::*, tasks::TaskPoolBuilder};
use crate::{channels::{outgoing::OutgoingOctetStringsAccessor, id::ChannelId, registry::ChannelRegistry, config::*}, octets::{octetstring::OctetString, varints::u24}, prelude::server::Client};
use super::{UdpClient, ports::PortBindings};

// TODO
// Despite the parallelism, this is pretty inefficient.
// It iterates over things when it doesn't need to several times.
// 21/08/2023 - now it iterates over even more things it doesn't need to!

pub(super) fn send_packets_system(
    registry: Res<ChannelRegistry>,
    channel_entities: Query<(&ChannelData, Option<&DirectionalChannel>, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    ports: Res<PortBindings>,
    mut clients: Query<(Entity, &UdpClient), With<Client>>,
    outgoing: OutgoingOctetStringsAccessor,
) {
    // Create task pool
    let pool = TaskPoolBuilder::new()
        .thread_name("UdpSendPacketsPool".to_string())
        .build();

    // Place query data into map of mutexes to allow mutation by multiple threads
    let mut query_mutex_map = BTreeMap::new();
    for (id, udp/*, seq*/) in clients.iter_mut() {
        query_mutex_map.insert(id, Mutex::new(udp));
    }

    // Intentional borrow to prevent moves
    let registry = &registry;
    let channel_entities = &channel_entities;
    let outgoing = &outgoing;
    let query_mutex_map = &query_mutex_map;

    // Create tasks for all ports
    pool.scope(|s| {
        for port in ports.iter() {
            s.spawn(async move {
                let (_, socket, clients) = port;
                let mut buffer = Vec::with_capacity(1500);

                // Take locks for our clients
                let mut locks = query_mutex_map.iter()
                    .filter(|(k,_)| clients.contains(k))
                    .map(|(k,v)| (k, v.lock().unwrap()))
                    .collect::<BTreeMap<_, _>>();

                for client in clients {
                    // Get client entity
                    let client_data = locks.get_mut(client).unwrap();

                    // Iterate all channels
                    let channels = outgoing.by_channel();
                    for channel in channels {
                        // Get channel data
                        let channel_id = channel.id();
                        let channel_ent = registry.get_from_id(channel_id)
                            .expect("Tried to send a packet to a channel that did not exist");
                        let channel_config = channel_entities.get(channel_ent)
                            .expect("Channel was in registry but the associated entity didn't exist");
                        let (channel_type_path, direction, ordered, reliable, fragmented) =
                            (channel_config.0.type_path(), channel_config.1, channel_config.2.is_none(), channel_config.3.is_some(), channel_config.4.is_some());

                        // The number sent over the network is shifted by 1 due to
                        // the fact that value 0 is reserved for messages sent by the transport layer
                        let channel_id_shift = (channel_id.0 + 1.into())
                            .expect("Too many channels! The transport layer imposes a restriction of 2^24-1.");

                        let sending_data = ChannelSendingData { reliable, _ordered: ordered, _fragmented: fragmented };

                        // Check channel direction
                        if direction.is_some_and(|v| *v == DirectionalChannel::ClientToServer) {
                            panic!("Tried to send a message on client to server channel: {}", channel_type_path);
                        }

                        // Iterate all octet strings
                        for (target, octets) in channel.strings().read() {
                            // Check this message is for this client
                            if target.excludes(*client) { continue }

                            // Send packet
                            send_octet_string(
                                &mut buffer,
                                socket,
                                channel_id_shift,
                                octets,
                                sending_data,
                                *client,
                                client_data
                            );
                        }
                    }

                    // Sanity check
                    // assert_eq!(highest_sequence_id, client_data.1.local_sequence);
                }
            });
        }
    });
}

// Data useful to `send_udp_packets`. Not necessary, but makes life easier.
#[derive(Clone, Copy)]
struct ChannelSendingData {
    pub reliable: bool,
    pub _ordered: bool,
    pub _fragmented: bool,
}

fn send_octet_string(
    buffer: &mut Vec<u8>,
    socket: &UdpSocket,
    channel: u24,
    octets: &OctetString,
    settings: ChannelSendingData,
    client_id: Entity,
    client_data: &mut MutexGuard<'_, &UdpClient>
) {
    // Write channel ID
    for octet in channel.bytes() { buffer.push(octet); }

    // Write highest sequence ID
    // for octet in highest.bytes() { udp_payload.push(octet);}

    // Write packet sequence ID for reliable channels
    // let mut sequence: SequenceId = 0.into(); // should get overwritten anyway
    // if settings.reliable {
    //     sequence = client_data.1.next();
    //     for octet in sequence.bytes() { udp_payload.push(octet); }
    // }

    // Write octet string
    if octets.as_slice().len() > 1500 - buffer.len() { panic!("Packet was too big. Fragmenting is not currently supported, try sending your data in multiple pieces."); }
    for octet in octets.as_slice() { buffer.push(*octet); }

    // Send data to remote peer
    socket.send_to(&buffer, client_data.address).unwrap();

    // // Store octet string for reliability
    // if !settings.reliable { return }
    // reliable.push((client_id, sequence, octets.clone()));
}