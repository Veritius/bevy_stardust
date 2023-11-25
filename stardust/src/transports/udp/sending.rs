use std::collections::BTreeMap;
use std::sync::Mutex;
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use crate::messages::incoming::IncomingMessageQueue;
use crate::messages::outgoing::TransportOutgoingReader;
use crate::prelude::*;
use super::established::UdpConnection;
use super::ports::PortBindings;

/// Sends octet strings using a taskpool strategy.
pub(super) fn send_packets_system(
    mut peers: Query<(Entity, &mut UdpConnection)>,
    registry: Res<ChannelRegistry>,
    ports: Res<PortBindings>,
    outgoing: TransportOutgoingReader,
) {
    // Create task pool
    let taskpool = TaskPoolBuilder::new()
        .thread_name("UDP pkt send".to_string())
        .build();

    // Create mutexes for all query entries, to allow mutation by multiple threads.
    // The mutexes are only to make the borrow checker happy, and are only locked once.
    let peer_locks = peers.iter_mut()
        .map(|x| (x.0, Mutex::new(x.1)))
        .collect::<BTreeMap<_,_>>();
    
    // Create a block just for variable shadowing only where we need it.
    {
        let ports = &ports;
        let peer_locks = &peer_locks;

        // Iterate over all messages
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
                            (*id, lock)
                        })
                        .collect::<BTreeMap<_, _>>();

                    // Iterate over all peers
                    for peer_id in peers {
                        let mut peer = peer_locks.get_mut(peer_id)
                            .expect("Todo: handle this case");
                    }
                });
            }
        });
    }
}