use std::net::{SocketAddr, UdpSocket};
use std::{collections::BTreeMap, sync::Mutex};
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use json::{JsonValue, object};
use once_cell::sync::Lazy;
use semver::Version;
use crate::channels::incoming::IncomingNetworkMessages;
use crate::prelude::*;
use crate::protocol::UniqueNetworkHash;
use crate::transports::udp::{TRANSPORT_LAYER_REQUIRE, TRANSPORT_LAYER_REQUIRE_STR};
use super::PACKET_MAX_BYTES;
use super::connections::{EstablishedUdpPeer, PendingUdpPeer};
use super::ports::PortBindings;

/// Processes packets from bound ports using a task pool strategy.
pub(super) fn receive_packets_system(
    mut commands: Commands,
    mut active_peers: Query<(Entity, &NetworkPeer, &mut EstablishedUdpPeer, &mut IncomingNetworkMessages)>,
    mut pending_peers: Query<(Entity, &mut PendingUdpPeer)>,
    registry: Res<ChannelRegistry>,
    channels: Query<(Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    ports: Res<PortBindings>,
    hash: Res<UniqueNetworkHash>,
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

                // Map addresses to entity IDs so we can lookup peers faster
                // TODO: Test performance without this. It's probably negligible.
                let addresses = active_locks
                    .iter()
                    .map(|(k, v)| (v.1.address, *k))
                    .chain(pending_locks.iter().map(|(k, v)| (v.address, *k)))
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

                    if let Some(entity) = addresses.get(&origin) {
                        // Assert they're only in one map otherwise it'd be weird.
                        debug_assert_ne!(active_locks.contains_key(entity), pending_locks.contains_key(entity));

                        if let Some(mut address) = active_locks.get(entity) {
                            // Active peer, process their packet normally
                            todo!()
                        } else if let Some(mut address) = pending_locks.get(entity) {
                            // Pending peer, let's see what they have to say
                            todo!()
                        }
                    } else {
                        // Unknown peer, likely reaching out to connect to us.
                        match process_zero_packet_from_unknown(
                            &buffer,
                            octets,
                            socket,
                            origin,
                            hash.hex(),
                            true, // TODO: Allow changing this
                        ) {
                            UnknownZeroPacketResult::Discarded => todo!(),
                            UnknownZeroPacketResult::AcceptUnknownRemote => todo!(),
                        }
                    }
                }
            });
        }
    });
}

fn process_zero_packet_from_known(
    buffer: &[u8; PACKET_MAX_BYTES],
    octets: usize,
) -> KnownZeroPacketResult {
    // Turn data into a JSON table
    // Will simply discard the packet if any of this fails
    let string = std::str::from_utf8(&buffer[3..octets]);
    let string = if string.is_err() { return KnownZeroPacketResult::Discarded } else { string.unwrap() };
    let json = json::parse(string);
    let json = if json.is_err() { return KnownZeroPacketResult::Discarded } else { json.unwrap() };

    // Get the response from the known peer
    let response = match &json["response"] {
        JsonValue::Short(val) => val.as_str(),
        JsonValue::String(val) => val.as_str(),
        _ => { return KnownZeroPacketResult::Discarded },
    };

    match response {
        "conn_accepted" => {
            match &json["response"] {
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
    buffer: &[u8; PACKET_MAX_BYTES],
    octets: usize,
    socket: &UdpSocket,
    address: SocketAddr,
    protocol: &str,
    allow_new: bool,
)  -> UnknownZeroPacketResult {
    // Turn data into a JSON table
    // Will simply discard the packet if any of this fails
    let string = std::str::from_utf8(&buffer[3..octets]);
    let string = if string.is_err() { return UnknownZeroPacketResult::Discarded } else { string.unwrap() };
    let json = json::parse(string);
    let json = if json.is_err() { return UnknownZeroPacketResult::Discarded } else { json.unwrap() };

    // Check transport version
    let transport = match &json["transport"] {
        JsonValue::Short(val) => val.as_str(),
        JsonValue::String(val) => val.as_str(),
        _ => { return UnknownZeroPacketResult::Discarded }
    };

    let mut split = transport.split('-');
    match split.next() {
        Some(val) => {
            if val != "udp" { return UnknownZeroPacketResult::Discarded }
        },
        None => { return UnknownZeroPacketResult::Discarded },
    }
    match split.next() {
        Some(val) => {
            // Check their version
            let version = val.parse::<Version>();
            let version = if version.is_err() { return UnknownZeroPacketResult::Discarded } else { version.unwrap() };
            if !TRANSPORT_LAYER_REQUIRE.matches(&version) {
                // Inform the remote peer of their incorrect version
                // Can't do the same thing as CLOSED_DENIED_PREPROC because if there were
                // different versions of the transport layer it could give an invalid response
                // that could be very confusing to anyone trying to debug
                let result = socket.send_to(object! {
                    "response": "conn_rejected",
                    "reason": format!("Transport layer version not accepted, requires {}", TRANSPORT_LAYER_REQUIRE_STR),
                }.dump().as_bytes(), address);
                if result.is_err() { error!("Error while sending a packet: {}", result.unwrap_err()); }
                return UnknownZeroPacketResult::Discarded
            }
        },
        None => { return UnknownZeroPacketResult::Discarded }
    }

    // Check request type
    let request = match &json["request"] {
        JsonValue::Short(val) => val.as_str(),
        JsonValue::String(val) => val.as_str(),
        _ => { return UnknownZeroPacketResult::Discarded }
    };

    return match request {
        "join" => {
            match process_join_zero_packet_from_unknown(
                &json,
                &socket,
                address,
                protocol,
                allow_new,
            ) {
                true => UnknownZeroPacketResult::AcceptUnknownRemote,
                false => UnknownZeroPacketResult::Discarded,
            }
        },
        _ => UnknownZeroPacketResult::Discarded,
    }
}

fn process_join_zero_packet_from_unknown(
    json: &JsonValue,
    socket: &UdpSocket,
    address: SocketAddr,
    protocol: &str,
    allow_new: bool
) -> bool {
    // Since this string remains constant across runtime, we initialise it once using a Lazy
    // This saves us having to allocate a String every time we want to tell the remote peer one of these things
    // Arguably the performance gains aren't even worth it, but whatever.
    static CLOSED_DENIED_PREPROC: Lazy<String> = Lazy::new(|| {
        object! {
            "response": "conn_rejected",
            "reason": "Connection closed to new peers",
        }.dump()
    });

    // Check their protocol
    let other_protocol = match &json["protocol"] {
        JsonValue::Short(val) => val.as_str(),
        JsonValue::String(val) => val.as_str(),
        _ => { return false }
    };

    if other_protocol != protocol {
        // Inform remote peer their transport value is incorrect
        let response = object! {
            "response": "conn_rejected",
            "reason": format!("Invalid protocol hash, server has {}", protocol),
        }.dump();
        let result = socket.send_to(response.as_bytes(), address);
        if result.is_err() { error!("Error while sending a packet: {}", result.unwrap_err()); }
        return false
    }

    // Check if new connections are allowed
    // This is done after the protocol check since IMO that's more important to whoever's joining
    if !allow_new {
        let result = socket.send_to(CLOSED_DENIED_PREPROC.as_bytes(), address);
        if result.is_err() { error!("Error while sending a packet: {}", result.unwrap_err()); }
        return false
    }

    // All checks succeeded - let them in!
    return true
}

/// The outcome of receiving a zero packet from an unknown peer.
enum UnknownZeroPacketResult {
    Discarded,
    Rejected,
    AcceptUnknownRemote,
}

fn process_game_packet(
    buffer: &[u8; PACKET_MAX_BYTES],
    octets: usize,
) {
    todo!()
}