use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{RwLock, Mutex};
use std::time::{Instant, Duration};
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use crate::messages::incoming::NetworkMessageStorage;
use crate::prelude::*;
use crate::protocol::ProtocolId;
use crate::scheduling::NetworkScheduleData;
use super::ordering::Ordering;
use super::outgoing::{OutgoingConnectionAttempts, OutgoingAttemptResult};
use super::reliability::Reliability;
use super::{TRANSPORT_IDENTIFIER, COMPAT_GOOD_VERSIONS};
use super::established::{AllowNewConnections, UdpConnection, Disconnected};
use super::ports::PortBindings;

/// Minimum amount of octets in a packet before it's ignored.
const MIN_OCTETS: usize = 5;

/// Processes packets from bound ports using a task pool strategy.
pub(super) fn receive_packets_system(
    mut commands: Commands,
    mut peers: Query<(Entity, &mut UdpConnection, Option<&mut NetworkMessageStorage>)>,
    mut attempts: ResMut<OutgoingConnectionAttempts>,
    schedule: NetworkScheduleData,
    registry: Res<ChannelRegistry>,
    mut ports: ResMut<PortBindings>,
    protocol: Res<ProtocolId>,
    allow_new: Res<AllowNewConnections>,
) {
    // Create task pool for parallel accesses
    let taskpool = TaskPoolBuilder::default()
        .thread_name("UDP pkt receive".to_string())
        .build();

    // Storage for outgoing connection attempts that have signalled acceptance or denial
    let outgoing_attempt_results: Mutex<Vec<(usize, OutgoingAttemptResult)>> = Mutex::default();

    // Synchronised set of addresses to prevent accepting the same peer twice
    // While this does block, the cost is nothing compared to syscalls and I/O
    let active_addresses: RwLock<Vec<SocketAddr>> = peers.iter()
        .map(|(_, x, _)| x.address)
        .collect::<Vec<_>>()
        .into();

    // Create mutexes for all query entries, to allow mutation by multiple threads.
    // The mutexes are only to make the borrow checker happy, and are only locked once.
    let peer_locks = peers.iter_mut()
        .map(|x| (x.0, Mutex::new((x.1, x.2))))
        .collect::<BTreeMap<_,_>>();

    // Put the threaded logic in a block so anything we only use in the taskpool is dropped
    // This isn't necessary, but it's convenient, so variables can be shadowed without issues later on
    {
        // Wrap some things in synchronisation primitives for _infrequent_ parallel access
        let commands = &Mutex::new(&mut commands);

        // Explicit borrows to prevent moves into the futures
        let ports = &ports;
        let protocol = &protocol;
        let active_addresses = &active_addresses;
        let outgoing_attempt_results = &outgoing_attempt_results;
        let peer_locks = &peer_locks;
        let attempts = &attempts;

        // Start reading bytes from all ports in parallel
        taskpool.scope(|s| {
            for (_, socket, peers) in ports.iter() {
                s.spawn(async move {
                    // Take locks from the mutex elements
                    let mut peer_locks = peers.iter()
                        .map(|id| {
                            let lock = match peer_locks.get(id).unwrap().try_lock() {
                                Ok(lock) => lock,
                                Err(error) => {
                                    // This is a panic because by all means this should never happen, and must be reported immediately
                                    // If this was an error log, it would probably spam the console a lot, and that's not really helpful
                                    // Also dumps the entirety of the PortBindings resource into the panic message for debugging purposes
                                    let dump = format!("{:?}", ports.iter().collect::<Vec<_>>());
                                    panic!("Peer data mutex was already locked or had already been locked when a task tried to access it: {error}. This should never happen!\nPortBindings dump is as follows: {dump}");
                                },
                            };
                            (lock.0.address, (*id, lock))
                        })
                        .collect::<BTreeMap<_, _>>();

                    let mut buffer = [0u8; 1472];
                    loop {
                        // Try to read a packet from the socket
                        let (octets_read, origin) = match socket.recv_from(&mut buffer) {
                            Ok(s) => s,
                            Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                            Err(ref e) if e.kind() == ErrorKind::Interrupted => break,
                            Err(ref e) if e.kind() == ErrorKind::ConnectionReset => continue,
                            Err(e) => {
                                // Actual I/O error, stop reading
                                error!("Error while reading packets: {e}");
                                break
                            },
                        };

                        // Check length of message
                        if octets_read < MIN_OCTETS { continue }

                        if let Some((identifier, data)) = peer_locks.get_mut(&origin) {
                            // Client is established as an entity already
                            todo!()
                        } else if active_addresses.read().unwrap().contains(&origin) {
                            // Client has just been accepted, this is probably an old message
                            todo!()
                        } else if let Some((index, _)) = attempts.get_attempt_from_address(&origin) {
                            // This packet is from a person we are trying to connect to
                            receive_packet_from_attempt_target(
                                &buffer[..octets_read],
                                origin,
                                index,
                                active_addresses,
                                outgoing_attempt_results,
                            )
                        } else {
                            // We don't know this person yet
                            receive_packet_from_unknown(
                                &buffer[..octets_read],
                                origin,
                                socket,
                                &*protocol,
                                active_addresses,
                                ports,
                                commands,
                            )
                        }
                    }
                });
            }
        });
    }

    for (index, result) in outgoing_attempt_results.lock().unwrap().iter() {
        todo!()
    }

    ports.commit_reservations();
}

fn receive_packet_from_attempt_target(
    data: &[u8],
    origin: SocketAddr,
    index: usize,
    active: &RwLock<Vec<SocketAddr>>,
    results: &Mutex<Vec<(usize, OutgoingAttemptResult)>>,
) {
    // Check their response type
    active.write().unwrap().push(origin);
    let mut lock = results.lock().unwrap();
    match data[0] {
        1 => {
            lock.push((index, OutgoingAttemptResult::Accepted {
                rel_idx: u16::from_be_bytes(data[1..3].try_into().unwrap()),
                port: u16::from_be_bytes(data[3..5].try_into().unwrap()),
            }))
        },
        2 => {
            lock.push((index, OutgoingAttemptResult::Rejected {
                reason: Disconnected::from(&data[1..])
            }))
        },
        _ => {
            lock.push((index, OutgoingAttemptResult::BadResponse));
            return
        }
    }
}

fn receive_packet_from_unknown(
    data: &[u8],
    origin: SocketAddr,
    socket: &UdpSocket,
    protocol: &ProtocolId,
    active: &RwLock<Vec<SocketAddr>>,
    ports: &PortBindings,
    commands: &Mutex<&mut Commands>,
) {
    // TODO: When iter_next_chunk is stabilised, use it here

    // Check packet length
    if data.len() < 23 {
        return;
    }

    // Message type
    if data[0] != 0 {
        todo!()
    }

    // Unique identifier for the transport layer
    if u64::from_be_bytes(data[1..9].try_into().unwrap()) != TRANSPORT_IDENTIFIER {
        todo!()
    }

    // Transport version integer
    if !COMPAT_GOOD_VERSIONS.contains(&u32::from_be_bytes(data[9..13].try_into().unwrap())) {
        todo!()
    }

    // Unique protocol hash
    if u64::from_be_bytes(data[13..21].try_into().unwrap()) != protocol.int() {
        todo!()
    }

    // All checks passed, create connection entity

    let mut reliability = Reliability::default();
    reliability.remote = u16::from_be_bytes(data[21..23].try_into().unwrap());
    reliability.local = fastrand::u16(..);
    let local = reliability.local.to_be_bytes();

    let ordering = Ordering::default();

    let id = commands.lock().unwrap().spawn((
        UdpConnection {
            address: origin,
            last_sent: None,
            last_received: None,
            timeout: Duration::from_secs(10),
            reliability,
            ordering,
        },

        NetworkPeer {
            connected: Instant::now(),
        },

        NetworkMessageStorage::new(),
    )).id();

    active.write().unwrap().push(origin.clone());

    // Send a response packet

    let mut buffer = [0u8; 5];
    buffer[0] = 1;
    buffer[1] = local[0];
    buffer[2] = local[1];

    let port = ports.make_reservation(id).to_be_bytes();
    buffer[3] = port[0];
    buffer[4] = port[1];

    socket.send_to(&buffer, origin)
        .expect("Failed to send acceptance packet, this shouldn't happen");
}