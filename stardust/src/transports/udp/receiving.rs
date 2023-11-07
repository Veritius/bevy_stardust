use std::net::{SocketAddr, UdpSocket};
use std::{collections::BTreeMap, sync::Mutex};
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use json::{JsonValue, object};
use once_cell::sync::Lazy;
use semver::Version;
use crate::messages::incoming::NetworkMessageStorage;
use crate::prelude::*;
use crate::protocol::UniqueNetworkHash;
use crate::scheduling::NetworkScheduleData;
use crate::transports::udp::{TRANSPORT_LAYER_REQUIRE, TRANSPORT_LAYER_REQUIRE_STR};
use super::PACKET_MAX_BYTES;
use super::connections::{EstablishedUdpPeer, PendingUdpPeer, AllowNewConnections};
use super::ports::{PortBindings, ReservationKey};
use super::reliability::Reliability;
use super::sending::send_zero_packet;

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
    
    // List of peers accepted this tick
    let accepted: Mutex<Vec<SocketAddr>> = Mutex::new(Vec::new());

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
    let ports_ref = ports.as_ref();
    let accepted = &accepted;
    let channel_map = &channel_map;
    let registry = &registry;
    let hash = &hash;
    let allow_new = &allow_new;

    // Process incoming packets
    let actions = taskpool.scope(|s| {
        for (_, socket, socket_peers) in ports.iter() {
            // Spawn task
            s.spawn(async move {
                // Storage for deferred actions this thread wants to make
                let mut actions = vec![];
                
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

                #[derive(Debug, Clone, Copy, PartialEq, Eq)]
                enum PeerType {
                    Active,
                    Pending,
                }

                // Map addresses to entity IDs so we can lookup peers faster
                // TODO: Test performance without this. It's probably negligible.
                let addresses = active_locks
                    .iter()
                    .map(|(k, v)| (v.1.address, (*k, PeerType::Active)))
                    .chain(
                        pending_locks
                        .iter()
                        .map(|(k, v)| (v.address, (*k, PeerType::Pending))))
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

                    if &buffer[0..3] == &[0u8; 3] {
                        if accepted.lock().unwrap().contains(&origin) { continue } // Skip this packet, we've already accepted them
                        if let Some((entity, ptype)) = addresses.get(&origin) {
                            if *ptype == PeerType::Active { continue } // we ignore messages from active peers
                            match process_zero_packet_from_known(
                                &buffer[3..octets],
                            ) {
                                KnownZeroPacketResult::Discarded => {
                                    info!("{origin}'s response to a connection attempt failed to parse, stopping connection");
                                    actions.push(DeferredAction::ShutdownPeer(*entity));
                                    continue;
                                },
                                KnownZeroPacketResult::Accepted(port) => {
                                    info!("Connection attempt to {origin} was accepted, reassigning to port {port}");
                                    actions.push(DeferredAction::MakeEstablished(*entity, port));
                                    accepted.lock().unwrap().push(origin);
                                    continue;
                                },
                                KnownZeroPacketResult::Rejected(reason) => {
                                    match reason {
                                        Some(val) => { info!("Connection attempt to {origin} was rejected: {val}"); },
                                        None => { info!("Connection attempt to {origin} was rejected: no reason given"); },
                                    }
                                    actions.push(DeferredAction::ShutdownPeer(*entity));
                                    continue;
                                },
                            }
                        } else {
                            match process_zero_packet_from_unknown(
                                &buffer[3..octets],
                                socket,
                                origin,
                                ports_ref,
                                hash.hex(),
                                allow_new.0,
                            ) {
                                UnknownZeroPacketResult::Discarded => { continue },
                                UnknownZeroPacketResult::Rejected => {
                                    info!("Connection attempt from {origin:?} was rejected");
                                    continue;
                                },
                                UnknownZeroPacketResult::Accepted(reskey) => {
                                    actions.push(DeferredAction::FinishReservation(reskey, origin));
                                    accepted.lock().unwrap().push(origin);
                                    continue;
                                },
                            }
                        }
                    } else {
                        if let Some((entity, ptype)) = addresses.get(&origin) {
                            if *ptype == PeerType::Pending { continue } // we ignore messages from pending peers
                            process_game_packet(&buffer, octets - 3)
                        } else {
                            // Don't know them, don't care about them
                            continue
                        }
                    }
                }

                // Return actions
                actions
            });
        }
    });

    // Apply deferred actions queued by tasks
    for action in actions.iter().flatten() {
        match action {
            DeferredAction::FinishReservation(key, addr) => {
                let entity = commands.spawn((
                    NetworkPeer::new(),
                    EstablishedUdpPeer {
                        address: *addr,
                        reliability: Reliability::default(),
                    },
                )).id();
                ports.take_reservations([(*key, entity)].iter().cloned());
            },
            DeferredAction::ShutdownPeer(peer) => {
                ports.remove_client(*peer);
                commands.entity(*peer).despawn();
            },
            DeferredAction::MakeEstablished(peer, port) => {
                let ip = pending_peers.get_mut(*peer).unwrap().1.address.ip();
                commands.entity(*peer).remove::<PendingUdpPeer>().insert(EstablishedUdpPeer {
                    address: SocketAddr::new(ip, *port),
                    reliability: Reliability::default(),
                });
            },
        }
    }

    #[cfg(debug_assertions="true")]
    ports.confirm_reservation_emptiness();
}

#[derive(Debug, Clone)]
enum DeferredAction {
    FinishReservation(ReservationKey, SocketAddr),
    ShutdownPeer(Entity),
    MakeEstablished(Entity, u16),
}

fn process_zero_packet_from_known(
    buffer: &[u8],
) -> KnownZeroPacketResult {
    // Turn data into a JSON table
    // Will simply discard the packet if any of this fails
    let string = std::str::from_utf8(&buffer);
    let string = if string.is_err() { return KnownZeroPacketResult::Discarded } else { string.unwrap() };
    let json = json::parse(string);
    let json = if json.is_err() { return KnownZeroPacketResult::Discarded } else { json.unwrap() };

    // Get the response from the known peer
    let response = match &json["msg"] {
        JsonValue::Short(val) => val.as_str(),
        JsonValue::String(val) => val.as_str(),
        _ => { return KnownZeroPacketResult::Discarded },
    };

    match response {
        "conn_accepted" => {
            match &json["use_port"] {
                JsonValue::Number(num) => {
                    let port = u16::try_from(*num);
                    if port.is_err() { return KnownZeroPacketResult::Discarded }
                    return KnownZeroPacketResult::Accepted(port.ok().unwrap())
                },
                _ => { return KnownZeroPacketResult::Discarded },
            }
        },
        "conn_rejected" => {
            let reason = match &json["reason"] {
                JsonValue::Null => return KnownZeroPacketResult::Rejected(None),
                JsonValue::Short(val) => val.as_str(),
                JsonValue::String(val) => val.as_str(),
                _ => { return KnownZeroPacketResult::Rejected(None) }
            };
            return KnownZeroPacketResult::Rejected(Some(reason.to_string()))
        },
        _ => { return KnownZeroPacketResult::Discarded }
    }
}

/// The outcome of receiving a zero packet from a known peer.
enum KnownZeroPacketResult {
    Discarded,
    Accepted(u16),
    Rejected(Option<String>),
}

fn process_zero_packet_from_unknown(
    buffer: &[u8],
    socket: &UdpSocket,
    address: SocketAddr,
    ports: &PortBindings,
    protocol: &str,
    allow_new: bool,
)  -> UnknownZeroPacketResult {
    // Turn data into a JSON table
    // Will simply discard the packet if any of this fails
    let string = std::str::from_utf8(&buffer);
    let string = if string.is_err() { return UnknownZeroPacketResult::Discarded } else { string.unwrap() };
    let json = json::parse(string);
    let json = if json.is_err() { return UnknownZeroPacketResult::Discarded } else { json.unwrap() };

    // Get transport version field
    let transport = match &json["transport"] {
        JsonValue::Short(val) => val.as_str(),
        JsonValue::String(val) => val.as_str(),
        _ => { return UnknownZeroPacketResult::Discarded }
    };

    // Check their version
    let version = transport.parse::<Version>();
    let version = if version.is_err() { return UnknownZeroPacketResult::Discarded } else { version.unwrap() };
    if !TRANSPORT_LAYER_REQUIRE.matches(&version) {
        // Inform the remote peer of their incorrect version
        // Can't do the same thing as CLOSED_DENIED_PREPROC because if there were
        // different versions of the transport layer it could give an invalid response
        // that could be very confusing to anyone trying to debug
        send_zero_packet(socket, address, object! {
            "msg": "conn_rejected",
            "reason": format!("Transport layer version not accepted, requires {}", TRANSPORT_LAYER_REQUIRE_STR),
        }.dump().as_bytes());
        return UnknownZeroPacketResult::Discarded
    }

    // Check request type
    let request = match &json["msg"] {
        JsonValue::Short(val) => val.as_str(),
        JsonValue::String(val) => val.as_str(),
        _ => { return UnknownZeroPacketResult::Discarded }
    };

    return match request {
        "req_join" => {
            return process_join_zero_packet_from_unknown(
                ports,
                &json,
                &socket,
                address,
                protocol,
                allow_new,
            )
        },
        _ => UnknownZeroPacketResult::Discarded,
    }
}

fn process_join_zero_packet_from_unknown(
    ports: &PortBindings,
    json: &JsonValue,
    socket: &UdpSocket,
    address: SocketAddr,
    protocol: &str,
    allow_new: bool
) -> UnknownZeroPacketResult {
    // Since this string remains constant across runtime, we initialise it once using a Lazy
    // This saves us having to allocate a String every time we want to tell the remote peer one of these things
    // Arguably the performance gains aren't even worth it, but whatever.
    static CLOSED_DENIED_PREPROC: Lazy<String> = Lazy::new(|| {
        object! {
            "msg": "conn_rejected",
            "reason": "Connection closed to new peers",
        }.dump()
    });

    // Check their protocol
    let other_protocol = match &json["protocol"] {
        JsonValue::Short(val) => val.as_str(),
        JsonValue::String(val) => val.as_str(),
        _ => { return UnknownZeroPacketResult::Rejected }
    };

    if other_protocol != protocol {
        // Inform remote peer their transport value is incorrect
        let response = object! {
            "msg": "conn_rejected",
            "reason": format!("Invalid protocol hash, server has {}", protocol),
        }.dump();
        send_zero_packet(socket, address, response.as_bytes());
        return UnknownZeroPacketResult::Rejected
    }

    // Check if new connections are allowed
    // This is done after the protocol check since IMO that's more important to whoever's joining
    if !allow_new {
        send_zero_packet(socket, address, CLOSED_DENIED_PREPROC.as_bytes());
        return UnknownZeroPacketResult::Rejected
    }

    // All checks succeeded, reserve their position and inform them of acceptance
    let (port, reskey) = ports.make_reservation();
    let response = object! {
        "msg": "conn_accepted",
        "use_port": port
    }.dump();
    send_zero_packet(socket, address, response.as_bytes());

    // Log result and return the reservation key
    info!("New peer on {address} accepted and assigned to {port}");
    return UnknownZeroPacketResult::Accepted(reskey)
}

/// The outcome of receiving a zero packet from an unknown peer.
enum UnknownZeroPacketResult {
    Discarded,
    Rejected,
    Accepted(ReservationKey),
}

fn process_game_packet(
    buffer: &[u8; PACKET_MAX_BYTES],
    octets: usize,
) {
    todo!()
}