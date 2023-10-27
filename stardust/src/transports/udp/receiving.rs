use std::net::{SocketAddr, UdpSocket};
use std::sync::RwLock;
use std::{collections::BTreeMap, sync::Mutex};
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use json::{JsonValue, object};
use once_cell::sync::Lazy;
use semver::Version;
use crate::connections::peer;
use crate::messages::incoming::NetworkMessageStorage;
use crate::prelude::*;
use crate::protocol::UniqueNetworkHash;
use crate::scheduling::NetworkScheduleData;
use crate::transports::udp::packet::PacketKind;
use crate::transports::udp::{TRANSPORT_LAYER_REQUIRE, TRANSPORT_LAYER_REQUIRE_STR};
use super::PACKET_MAX_BYTES;
use super::connections::{EstablishedUdpPeer, PendingUdpPeer, AllowNewConnections};
use super::ports::{PortBindings, ReservationKey};

/// Processes packets from bound ports using a task pool strategy.
pub(super) fn receive_packets_system(
    mut commands: Commands,
    mut active_peers: Query<(Entity, &NetworkPeer, &mut EstablishedUdpPeer, &mut NetworkMessageStorage)>,
    mut pending_peers: Query<(Entity, &mut PendingUdpPeer)>,
    schedule: NetworkScheduleData,
    registry: Res<ChannelRegistry>,
    channels: Query<(Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    mut ports: ResMut<PortBindings>,
    hash: Res<UniqueNetworkHash>,
    allow_new: Res<AllowNewConnections>,
) {
    // Create task pool for parallel accesses
    let taskpool = TaskPoolBuilder::default()
        .thread_name("UDP pkt receive".to_string())
        .build();

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    enum PeerType {
        Established,
        Pending,
    }

    // List of addresses we know about, whether assigned to a peer or not
    // This might be due to them already being connected or they've been accepted already
    // This shouldn't hurt performance too much, since most of the cost is syscalls anyway
    let known_addresses = RwLock::new(
        active_peers.iter().map(|(_, _, e, _)| (e.address, PeerType::Established))
        .chain(pending_peers.iter().map(|(_, e)| (e.address, PeerType::Pending)))
        .collect::<BTreeMap<_, _>>()
    );

    // Place query data into map of mutexes to allow mutation by multiple threads
    // This doesn't block since each key-value pair will only be accessed by one thread each.
    let active_peers_map = active_peers
        .iter_mut()
        .map(|(id, client, udp, incoming)| {
            (id, Mutex::new((client, udp, incoming)))
        })
        .collect::<BTreeMap<_, _>>();
    let pending_peers_map = pending_peers
        .iter_mut()
        .map(|(id, pending)| {
            (id, Mutex::new(pending))
        })
        .collect::<BTreeMap<_, _>>();

    // Map of channels to speed up accesses
    let channel_map = (0..registry.channel_count())
        .map(|v| ChannelId::try_from(v).unwrap())
        .map(|v| {
            let ent = registry.get_from_id(v).unwrap();
            let q = channels.get(ent).unwrap();
            (v, (q.0.is_some(), q.1.is_some(), q.2.is_some()))
        })
        .collect::<BTreeMap<ChannelId, _>>();

    // Explicit borrows to prevent moves
    let active_peers_map = &active_peers_map;
    let pending_peers_map = &pending_peers_map;
    let known_addresses = &known_addresses;
    let ports_ref = ports.as_ref();
    let channel_map = &channel_map;
    let registry = &registry;
    let hash = &hash;
    let allow_new = &allow_new;

    // Process incoming packets
    taskpool.scope(|s| {
        for (_, socket, socket_peers) in ports.iter() {
            // Spawn task
            s.spawn(async move {                
                // Allocate a buffer for storing incoming data
                let mut buffer = [0u8; PACKET_MAX_BYTES];

                // Lock mutexes for our port-associated clients
                // This never blocks since each client is only accessed by one task at a time
                // but it still lets us mutate the client's components
                let mut active_locks = active_peers_map.iter()
                    .filter(|(k,_)| socket_peers.contains(k))
                    .map(|(k,v)| (*k, v.lock().unwrap()))
                    .collect::<BTreeMap<_, _>>();
                let mut pending_locks = pending_peers_map.iter()
                    .filter(|(k,_)| socket_peers.contains(k))
                    .map(|(k,v)| (*k, v.lock().unwrap()))
                    .collect::<BTreeMap<_, _>>();

                // Read all packets from the socket
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

                    let packet_kind = if let Ok(v) = PacketKind::try_from(buffer[0]) { v } else { continue };
                    let lock = known_addresses.read().unwrap();
                    let peer_type = lock.get(&origin);

                    // Route to the correct function for processing
                    match (peer_type, packet_kind) {
                        // A new connection attempt from a remote peer
                        (None, PacketKind::ConnectionManagement) => todo!(),
                        (Some(peer_type), PacketKind::ConnectionManagement) => {
                            match peer_type {
                                // An established peer managing their connection
                                PeerType::Established => {
                                    todo!()
                                },
                                // A pending peer trying to establish a connection
                                PeerType::Pending => {
                                    todo!()
                                },
                            }
                        },
                        // An established peer trying to send us game data
                        (Some(PeerType::Established), packet_kind) => {
                            match packet_kind {
                                PacketKind::ConnectionManagement => panic!(),
                                PacketKind::SingleMessage => {
                                    todo!()
                                },
                            }
                        },
                        _ => { continue }
                    }
                }
            });
        }
    });

    #[cfg(debug_assertions="true")]
    ports.confirm_reservation_emptiness();
}