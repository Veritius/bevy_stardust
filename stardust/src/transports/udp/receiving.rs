use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::{RwLock, Mutex};
use bevy::ecs::system::CommandQueue;
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use crate::messages::incoming::NetworkMessageStorage;
use crate::prelude::*;
use crate::protocol::UniqueNetworkHash;
use crate::scheduling::NetworkScheduleData;
use super::connections::{AllowNewConnections, UdpConnection};
use super::parallel::DeferredCommandQueue;
use super::ports::PortBindings;

/// Minimum amount of octets in a packet before it's ignored.
const MIN_OCTETS: usize = 3;

/// Processes packets from bound ports using a task pool strategy.
pub(super) fn receive_packets_system(
    mut commands: Commands,
    mut peers: Query<(Entity, &mut UdpConnection, Option<&mut NetworkMessageStorage>)>,
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

    // Put the threaded logic in a block so variable shadowing only happens inside it
    // This isn't necessary, but it's convenient, so ports doesn't have to be named something else
    // and would potentially be used after the threaded logic, which would cause problems
    {
        // Explicit borrows to prevent moves into the futures
        let ports = &ports;
        let active_addresses = &active_addresses;
        let peer_locks = &peer_locks;

        // Start reading bytes from all ports in parallel
        let mut task_commands = taskpool.scope(|s| {
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
                    let mut commands = CommandQueue::default();
                    let mut new: Vec<UdpConnection> = vec![];
                    let mut buffer = [0u8; 1472];

                    loop {
                        // Try to read a packet from the socket
                        let (octets_read, origin) = match socket.recv_from(&mut buffer) {
                            Ok(s) => s,
                            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                                // We've run out of packets to read
                                break
                            },
                            Err(e) => {
                                // Actual I/O error, stop reading
                                error!("Error while reading packets: {e}");
                                break
                            },
                        };

                        // Check length of message
                        if octets_read < MIN_OCTETS { continue }

                        // Get information about our peer
                        let origin_is_known = active_addresses.read().unwrap().contains(&origin);
                    }

                    // Return deferred mutations that this task wants to perform
                    return commands
                });
            }
        });

        // Add commands from tasks to Commands
        for command in task_commands.drain(..) {
            commands.add(DeferredCommandQueue(command))
        }
    }

    #[cfg(debug_assertions="true")]
    ports.confirm_reservation_emptiness();
}