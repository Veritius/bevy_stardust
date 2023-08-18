use std::{sync::{Mutex, MutexGuard}, net::UdpSocket, collections::BTreeMap};
use bevy::{prelude::*, tasks::TaskPool};
use crate::{shared::{channels::{outgoing::OutgoingOctetStringsAccessor, id::ChannelId}, octetstring::OctetString}, server::{clients::Client, prelude::*}};
use super::{UdpClient, ports::PortBindings, acks::ClientSequenceData};

// TODO
// Despite the parallelism, this is pretty inefficient.
// It iterates over things when it doesn't need to several times.

pub(super) fn send_packets_system(
    registry: Res<ChannelRegistry>,
    channel_entities: Query<(&ChannelData, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    ports: Res<PortBindings>,
    mut clients: Query<(Entity, &UdpClient, &mut ClientSequenceData), With<Client>>,
    outgoing: OutgoingOctetStringsAccessor,
) {
    // Create task pool
    let pool = TaskPool::new();

    // Place query data into map of mutexes to allow mutation by multiple threads
    let mut query_mutex_map = BTreeMap::new();
    for (id, udp, seq) in clients.iter_mut() {
        query_mutex_map.insert(id, Mutex::new((udp, seq)));
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
                        let (channel_type_path, channel_config, ordered, reliable, fragmented) =
                            (channel_config.0.type_path(), channel_config.0.config(), channel_config.1.is_none(), channel_config.2.is_some(), channel_config.3.is_some());

                        // Check channel direction
                        if channel_config.direction == ChannelDirection::ClientToServer {
                            panic!("Tried to send a message on client to server channel: {}", channel_type_path);
                        }

                        // Iterate all octet strings
                        for (target, octets) in channel.strings().read() {
                            // Check this message is for this client
                            if target.excludes(*client) { continue }

                            // Send packet
                            send_udp_packet(
                                socket,
                                channel_id,
                                octets,
                                client_data
                            );
                        }
                    }
                }
            });
        }
    });
}

fn send_udp_packet(
    socket: &UdpSocket,
    channel: ChannelId,
    octets: &OctetString,
    client_data: &mut MutexGuard<'_, (&UdpClient, Mut<'_, ClientSequenceData>)>
) {
    let mut udp_payload = Vec::with_capacity(1500);
    for octet in channel.bytes() { udp_payload.push(octet); }
    for octet in octets.as_slice() { udp_payload.push(*octet); }
    socket.send_to(&udp_payload, client_data.0.address).unwrap();
}