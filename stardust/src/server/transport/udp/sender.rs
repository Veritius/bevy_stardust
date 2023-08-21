use std::{sync::{Mutex, MutexGuard}, net::UdpSocket, collections::BTreeMap};
use bevy::{prelude::*, tasks::TaskPool};
use rand::Rng;
use crate::{shared::{channels::{outgoing::OutgoingOctetStringsAccessor, id::ChannelId}, octetstring::OctetString, reliability::{PeerSequenceData, SequenceId}, integers::u24}, server::{clients::Client, prelude::*}};
use super::{UdpClient, ports::PortBindings};

// TODO
// Despite the parallelism, this is pretty inefficient.
// It iterates over things when it doesn't need to several times.
// 21/08/2023 - now it iterates over even more things it doesn't need to!

pub(super) fn send_packets_system(
    registry: Res<ChannelRegistry>,
    channel_entities: Query<(&ChannelData, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    ports: Res<PortBindings>,
    mut clients: Query<(Entity, &UdpClient, &mut PeerSequenceData), With<Client>>,
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

                // Store sent packets for reliability
                let mut sent_packets = vec![];

                for client in clients {
                    // Get client entity
                    let client_data = locks.get_mut(client).unwrap();

                    // Count the amount of reliable messages we're going to be sending
                    let mut reliable_amount: usize = 0;
                    for channel in outgoing.by_channel() {
                        // Check if this channel is marked reliable
                        if channel_entities.get(registry.get_from_id(channel.id()).unwrap()).unwrap().2.is_none() { continue; }
                        for (target, _) in channel.read() {
                            if target.excludes(*client) { continue; }
                            reliable_amount += 1;
                        }
                    }

                    // Get the highest sequence id that will be sent, for reliability purposes
                    let highest_sequence_id = client_data.1.local_sequence.wrapping_add(TryInto::<u24>::try_into(reliable_amount).unwrap());

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
                        
                        let sending_data = ChannelSendingData { reliable, ordered, fragmented };

                        // Check channel direction
                        if channel_config.direction == ChannelDirection::ClientToServer {
                            panic!("Tried to send a message on client to server channel: {}", channel_type_path);
                        }

                        // Iterate all octet strings
                        for (target, octets) in channel.strings().read() {
                            // Check this message is for this client
                            if target.excludes(*client) { continue }

                            // Send packet
                            send_octet_string(
                                socket,
                                channel_id,
                                octets,
                                sending_data,
                                highest_sequence_id,
                                &mut sent_packets,
                                *client,
                                client_data
                            );
                        }
                    }
                }
            });
        }
    });
}

// Data useful to `send_udp_packets`. Not necessary, but makes life easier.
#[derive(Clone, Copy)]
struct ChannelSendingData {
    pub reliable: bool,
    pub ordered: bool,
    pub fragmented: bool,
}

fn send_octet_string(
    socket: &UdpSocket,
    channel: ChannelId,
    octets: &OctetString,
    settings: ChannelSendingData,
    highest: u24,
    reliable: &mut Vec<(Entity, SequenceId, OctetString)>,
    client_id: Entity,
    client_data: &mut MutexGuard<'_, (&UdpClient, Mut<'_, PeerSequenceData>)>
) {
    // Allocate vec for storing payload data
    let mut udp_payload = Vec::with_capacity(1500);

    // Write channel ID
    for octet in channel.bytes() { udp_payload.push(octet); }

    // Write highest sequence ID
    for octet in highest.bytes() { udp_payload.push(octet);}

    // Write packet sequence ID for reliable channels
    let mut sequence: SequenceId = 0.into(); // should get overwritten anyway
    if settings.reliable {
        sequence = client_data.1.next();
        for octet in sequence.bytes() { udp_payload.push(octet); }
    }

    // Write octet string
    if octets.as_slice().len() > 1500 - udp_payload.len() { panic!("Packet was too big. Fragmenting is not currently supported, try sending your data in multiple pieces."); }
    for octet in octets.as_slice() { udp_payload.push(*octet); }

    // Send data to remote peer
    if rand::thread_rng().gen_range(0.0..1.0) > 0.2 {
        socket.send_to(&udp_payload, client_data.0.address).unwrap();
    } else {
        info!("Intentionally failed to send a packet.");
    }

    // Store octet string for reliability
    if !settings.reliable { return }
    reliable.push((client_id, sequence, octets.clone()));
}