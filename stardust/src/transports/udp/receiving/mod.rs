mod incoming;
mod outgoing;
mod established;

use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::{RwLock, Mutex};
use std::time::Duration;
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use crate::messages::incoming::NetworkMessageStorage;
use crate::prelude::*;
use crate::protocol::ProtocolId;
use crate::scheduling::NetworkScheduleData;
use super::connections::{AllowNewConnections, UdpConnection, ConnectionStatus};
use super::ports::PortBindings;

/// Minimum amount of octets in a packet before it's ignored.
const MIN_OCTETS: usize = 3;

/// Processes packets from bound ports using a task pool strategy.
pub(super) fn receive_packets_system(
    mut commands: Commands,
    mut peers: Query<(Entity, &mut UdpConnection, Option<&mut NetworkMessageStorage>)>,
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
        let peer_locks = &peer_locks;

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

                    // Some variables the future uses
                    let mut new: Vec<UdpConnection> = vec![];
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

                        // Get or create a new UdpConnection object based on the origin address
                        let connection = match peer_locks.get_mut(&origin) {
                            Some((_, mutex)) => mutex.0.as_mut(),
                            None => {
                                match new.iter_mut().find(|p| p.address == origin) {
                                    Some(v) => v,
                                    None => {
                                        // Check this hasn't already been registered in another thread
                                        if active_addresses.read().unwrap().contains(&origin) { continue }
                                        active_addresses.write().unwrap().push(origin);

                                        // Add to this thread-local storage
                                        new.push(UdpConnection::new_incoming(origin, Duration::from_secs(30)));
                                        let idx = new.len();
                                        new.get_mut(idx).unwrap()
                                    },
                                }
                            },
                        };

                        // Process the packet
                        let message = &buffer[..octets_read];
                        match &mut connection.status {
                            ConnectionStatus::PendingIncoming(incoming) =>
                                incoming::process_pending_incoming(message, connection, protocol.int(), &*ports),
                            ConnectionStatus::PendingOutgoing(outgoing) =>
                                outgoing::process_pending_outgoing(message, connection),
                            ConnectionStatus::Established(established) =>
                                established::process_established(message, connection),
                            ConnectionStatus::Disconnected(_) =>
                                todo!(),
                        }
                    }

                    // Create entities for new connections
                    let mut commands = commands.lock().unwrap();
                    for new in new.drain(..) {
                        commands.spawn(new);
                    }
                });
            }
        });
    }
}