use std::net::{SocketAddr, UdpSocket};
use std::time::Instant;
use std::{collections::BTreeMap, sync::Mutex};
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use json::{JsonValue, object};
use semver::Version;
use crate::channels::incoming::IncomingNetworkMessages;
use crate::prelude::*;
use crate::prelude::server::Client;
use crate::protocol::UniqueNetworkHash;
use crate::transports::udp::{TRANSPORT_LAYER_REQUIRE, TRANSPORT_LAYER_REQUIRE_STR};
use super::pending::PendingConnection;
use super::{PACKET_HEADER_SIZE, PACKET_MAX_BYTES, UdpTransportState};
use super::peer::EstablishedUdpPeer;
use super::ports::PortBindings;

/// Processes packets from bound ports using a task pool strategy.
pub(super) fn receive_packets_system(
    mut commands: Commands,
    mut active_peers: Query<(Entity, &NetworkPeer, &mut EstablishedUdpPeer, &mut IncomingNetworkMessages)>,
    pending_peers: Query<(Entity, &PendingConnection)>,
    registry: Res<ChannelRegistry>,
    channels: Query<(Option<&DirectionalChannel>, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    ports: Res<PortBindings>,
    hash: Res<UniqueNetworkHash>,
) {
    // Create task pool for parallel accesses
    let taskpool = TaskPoolBuilder::default()
        .thread_name("UDP pkt receive".to_string())
        .build();

    // Storage for adding new clients
    let new_clients: Mutex<Vec<(u16, SocketAddr)>> = Mutex::new(Vec::new());

    // Place query data into map of mutexes to allow mutation by multiple threads
    // This doesn't block since each key-value pair will only be accessed by one thread each.
    let mut query_mutex_map = BTreeMap::new();
    for (id, client, udp, incoming) in active_peers.iter_mut() {
        query_mutex_map.insert(id, Mutex::new((client, udp, incoming)));
    }

    // Map of channels to speed up accesses
    let channel_map = (0..registry.channel_count())
        .map(|v| ChannelId::try_from(v).unwrap())
        .map(|v| {
            let ent = registry.get_from_id(v).unwrap();
            let q = channels.get(ent).unwrap();
            (v, (q.0, q.1.is_some(), q.2.is_some(), q.3.is_some()))
        })
        .collect::<BTreeMap<ChannelId, _>>();

    // Explicit borrows to prevent moves
    let new_clients = &new_clients;
    let query_mutex_map = &query_mutex_map;
    let channel_map = &channel_map;
    let registry = &registry;
    let hash = &hash;

    // Process incoming packets
    taskpool.scope(|s| {
        for (_, socket, socket_peers) in ports.iter() {
            // Spawn task
            s.spawn(async move {
                // Allocate a buffer for storing incoming data
                let mut buffer = [0u8; PACKET_MAX_BYTES];

                // Lock mutexes for our port-associated clients
                let mut locks = query_mutex_map.iter()
                    .filter(|(k,_)| socket_peers.contains(k))
                    .map(|(k,v)| (k, v.lock().unwrap()))
                    .collect::<BTreeMap<_, _>>();

                // Create vec of addresses for rapid filtering
                let addresses = locks.iter()
                    .map(|(_, guard)| guard.1.address)
                    .collect::<Vec<_>>();

                // Map addresses to entity IDs
                let address_map = locks.iter()
                    .map(|(entity, guard)| (guard.1.address, **entity))
                    .collect::<BTreeMap<_, _>>();

                // Read all packets from the socket
                let mut buffer = [0u8; PACKET_MAX_BYTES];
                loop {
                    // Read a packet
                    let (octets, origin) = match socket.recv_from(&mut buffer) {
                        Ok(n) => n,
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // No more data to read
                            break
                        }
                        Err(e) => {
                            // Something went wrong
                            error!("IO error while reading UDP socket {:?}: {}", socket.local_addr().unwrap(), e);
                            continue
                        },
                    };

                    // Check packet size
                    // If it's less than 4 bytes it isn't worth processing
                    if octets <= 3 { continue; }

                    // Start processing the message
                    match &buffer[0..2] == &[0; 3] {
                        // Zero packet - someone's trying to join
                        true => process_zero_packet(

                        ),
                        // Normal packet - probably from existing peer
                        false => process_game_packet(

                        ),
                    }
                }
            });
        }
    });
}

fn process_zero_packet(

) {

}

fn process_game_packet(

) {

}