use std::{collections::BTreeMap, net::UdpSocket};
use bevy::{prelude::*, tasks::TaskPool};
use crate::{server::clients::Client, shared::{channels::{components::*, incoming::IncomingNetworkMessages, registry::ChannelRegistry, id::ChannelId}, payload::{Payloads, Payload}}};
use super::{PACKET_HEADER_SIZE, MAX_PACKET_LENGTH, UdpClient};

/// Unfiltered socket for listening to UDP packets from unregistered peers.
#[derive(Resource)]
pub(super) struct UdpListener(pub UdpSocket);

impl UdpListener {
    pub fn new(port: u16) -> Self {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port))
            .expect("Couldn't bind to port");
        
        socket.set_nonblocking(true).unwrap();

        Self(socket)
    }
}

pub(super) fn receive_packets_system(
    mut clients: Query<(Entity, &Client, &UdpClient, &mut IncomingNetworkMessages)>,
    listener: Res<UdpListener>,
    channels: Query<(&ChannelData, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    channel_registry: Res<ChannelRegistry>,
) {
    // Create thread pool for processing
    let pool = TaskPool::new();

    // Explicitly borrow to prevent moves
    let channels = &channels;
    let channel_registry = &channel_registry;

    // Receive packets from clients in parallel
    let mut outputs = pool.scope(|s| {
        for (client_id, _, client_udp, client_incoming) in clients.iter_mut() {
            let client_id = client_id.clone();
            s.spawn(async move {
                let mut map = BTreeMap::new();
                let mut buffer = [0u8; MAX_PACKET_LENGTH];

                let addr = client_udp.address.clone();

                // Read all packets
                loop {
                    if let Ok(octets) = client_udp.socket.recv(&mut buffer) {
                        // Discard packet, too small to be useful.
                        if octets <= 3 { continue; }

                        // Get channel ID and check it exists
                        let channel_id = ChannelId::from_bytes(&buffer[0..=3].try_into().unwrap());
                        if !channel_registry.channel_exists(channel_id) { break; }

                        // Copy octets from buffer
                        let idx = octets - PACKET_HEADER_SIZE - 1;
                        let mut packet = Vec::with_capacity(idx);
                        for i in (PACKET_HEADER_SIZE + 1)..idx {
                            packet.push(buffer[i]);
                        }

                        map.entry(channel_id).or_insert(Vec::with_capacity(1)).push(packet);
                    } else {
                        // We're done reading packets
                        break;
                    }
                }

                // Process map into Payloads
                let mut nmap = BTreeMap::new();
                while map.len() != 0 {
                    let (cid, mut vec) = map.pop_first().unwrap();
                    let payloads = vec
                        .drain(..)
                        .map(|x| {
                            Payload::new(0, 0, x) })
                        .collect::<Vec<Payload>>();
                    nmap.insert(cid, payloads);
                }

                (client_id, nmap)
            });
        }
    });

    // Write client data
    for (client, map) in outputs.iter_mut() {
        let (_, _, _, mut data) = clients.get_mut(*client).unwrap();

        let keys = map.keys().cloned().collect::<Vec<ChannelId>>();
        let mut nmap = BTreeMap::new();
        for key in keys {
            let mut val = map.remove(&key).unwrap();
            val.shrink_to_fit();
            let val = val.into_boxed_slice();
            nmap.insert(key, val.into());
        }

        data.0 = nmap;
    }
}