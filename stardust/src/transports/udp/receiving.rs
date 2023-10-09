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
use super::{PACKET_HEADER_SIZE, PACKET_MAX_BYTES, UdpTransportState};
use super::peer::UdpPeer;
use super::ports::PortBindings;

/// Receives octet strings using a taskpool strategy.
pub(super) fn receive_packets_system(
    mut commands: Commands,
    mut peers: Query<(Entity, &NetworkPeer, &mut UdpPeer, &mut IncomingNetworkMessages)>,
    state: Res<State<UdpTransportState>>,
    ports: Option<ResMut<PortBindings>>,
    channels: Query<(Option<&DirectionalChannel>, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    registry: Res<ChannelRegistry>,
    hash: Res<UniqueNetworkHash>,
) {
    // Check optional resources
    if ports.is_none() { return; }
    let mut ports = ports.unwrap();

    // Create task pool
    let pool = TaskPoolBuilder::new()
        .thread_name("UdpReadPacketsPool".to_string())
        .build();

    // Storage for adding new clients
    let new_clients: Mutex<Vec<(u16, SocketAddr)>> = Mutex::new(Vec::new());

    // Place query data into map of mutexes to allow mutation by multiple threads
    let mut query_mutex_map = BTreeMap::new();
    for (id, client, udp, incoming) in peers.iter_mut() {
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
    let state = state.get();
    let new_clients = &new_clients;
    let query_mutex_map = &query_mutex_map;
    let channel_map = &channel_map;
    let registry = &registry;
    let hash = &hash;

    // Process incoming packets
    pool.scope(|s| {
        for (port, socket, socket_peers) in ports.iter() {
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

                // Read packets until we run out
                loop {
                    // Read packet
                    let (octets_read, from_address) = match socket.recv_from(&mut buffer) {
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
                    if octets_read < 3 { continue } // Packet is too small to be of any value

                    // Check channel id
                    let channel_id = ChannelId::from(TryInto::<[u8; 3]>::try_into(&buffer[..3]).unwrap());

                    // Process zero packet if a server
                    if channel_id.0 == 0.into() && *state == UdpTransportState::Server {
                        // Connected peers shouldn't be sending these packets
                        if addresses.contains(&from_address) {
                            let entity_id: &Entity = address_map.get(&from_address).unwrap();
                            let guard = locks.get_mut(entity_id).unwrap();
                            guard.1.hiccups += 1;
                        }

                        // Actually read their packet
                        let accepted = server_read_zero_packet(
                            &socket,
                            &from_address,
                            &buffer,
                            octets_read,
                            &hash.hex(),
                        );

                        // Add client
                        if accepted {
                            new_clients.lock().unwrap().push((port, from_address.clone()));
                        }
                    }

                    // Check address and header size again
                    if !addresses.contains(&from_address) { continue } // Not from a client associated with this socket
                    if octets_read < PACKET_HEADER_SIZE { continue } // Too small to be a game data packet

                    // Get channel config
                    let channel_id = ChannelId(channel_id.0 - 1.into()); // Shift the channel ID back
                    if !registry.channel_exists(channel_id) { continue } // Channel doesn't exist
                    let (direction, _ordered, _reliable, _fragmented) = channel_map.get(&channel_id).unwrap();

                    // Check channel direction
                    if direction.is_some_and(|v| *v == DirectionalChannel::ServerToClient) {
                        // Packet went in the wrong direction
                        let entity_id = address_map.get(&from_address).unwrap();
                        let peer = &mut locks.get_mut(entity_id).unwrap().1;
                        peer.hiccups += 1;
                        continue
                    }

                    // Get client lock
                    let entity_id: &Entity = address_map.get(&from_address).unwrap();
                    let guard = locks.get_mut(entity_id).unwrap();

                    // Any bytes that don't need to be cloned
                    let cutoff = PACKET_HEADER_SIZE;

                    // Copy data to vec and make Payload
                    let mut payload = Vec::with_capacity(octets_read - cutoff);
                    for oct in &buffer[cutoff..=octets_read] { payload.push(*oct); }
                    let payload = Payload::new(0, 0, payload);

                    // Place payload in incoming component
                    guard.2.append(channel_id, payload);
                }
            });
        }
    });

    // Add new clients
    for (idx, address) in new_clients.lock().unwrap().iter() {
        let entity = commands.spawn((
            NetworkPeer { connected: Instant::now() },
            UdpPeer {
                address: address.clone(),
                hiccups: 0,
            },
            Client,
        )).id();
        let new_port = ports.add_client(entity);
        let socket = ports.port(*idx).unwrap().0;
        send_bytes(socket, address, object! {
            "response": "accepted",
            "port": new_port,
        }.dump().as_bytes());
    }
}

/// Processes an auth packet (channel 0) and returns whether or not they should be accepted.
/// Will send back a message of denial by itself if the packet itself is valid.
fn server_read_zero_packet(
    socket: &UdpSocket,
    origin: &SocketAddr,
    buffer: &[u8; PACKET_MAX_BYTES],
    octets: usize,
    hash: &str,
) -> bool {
    // Parse received bytes into a JSON document
    // If any of these checks fail, we simply discard their auth attempt.
    let string = std::str::from_utf8(&buffer[3..octets]);
    let string = if string.is_err() { return false; } else { string.unwrap() };
    let json = json::parse(string);
    let json = if json.is_err() { return false; } else { json.unwrap() };

    // Check request field
    // If this fails we also discard their packet
    let request = match &json["request"] {
        json::JsonValue::Short(val) => val.as_str(),
        json::JsonValue::String(val) => val.as_str(),
        _ => { return false; }
    };

    // Match their request field
    // Currently there's only join, but other requests may be added
    // This exists despite only one value to make DoS amplification (by IP spoofing) attacks that much harder
    match request {
        "join" => {},
        _ => { return false; }
    }

    // Check layer version
    let layer_version = match &json["layer_version"] {
        json::JsonValue::Short(val) => val.as_str(),
        json::JsonValue::String(val) => val.as_str(),
        _ => { return false; }
    }.parse::<Version>();
    // If this fails to parse, discard it silently.
    let layer_version = if layer_version.is_err() { return false; } else { layer_version.unwrap() };

    if !TRANSPORT_LAYER_REQUIRE.matches(&layer_version) {
        // Inform the client that they have a bad version
        let json = object! {
            "response": "wrong_transport_version",
            "requires": TRANSPORT_LAYER_REQUIRE_STR,
        }.dump();
        send_bytes(socket, origin, json.as_bytes());
        return false;
    }

    // Check protocol ID
    let protocol_id = match &json["pid"] {
        JsonValue::Short(val) => val.as_str(),
        JsonValue::String(val) => val.as_str(),
        _ => { return false; }
    };
    if protocol_id != hash {
        // Inform the client that their protocol ID is different
        let json = object! {
            "response": "wrong_pid",
            "srv_pid": hash,
        }.dump();
        send_bytes(socket, origin, json.as_bytes());
        return false;
    }

    // All checks passed
    return true;
}

/// Send `bytes` to `origin`. Ignores errors.
fn send_bytes(socket: &UdpSocket, origin: &SocketAddr, bytes: &[u8]) {
    let _ = socket.send_to(bytes, origin);
}