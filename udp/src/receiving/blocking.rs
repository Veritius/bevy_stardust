use std::{sync::Mutex, collections::BTreeMap};
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{MAXIMUM_TRANSPORT_UNITS, established::UdpConnection, ports::BoundSocketManager, policy::BlockingPolicy};

pub(crate) fn blocking_receive_packets_system(
    registry: Res<ChannelRegistry>,
    sockets: Res<BoundSocketManager>,
    policy: Res<BlockingPolicy>,
    mut peers: Query<(Entity, &mut NetworkPeer, &mut UdpConnection)>,
    mut incoming: NetworkIncomingWriter,
) {
    // Mutexes to make the borrow checker happy
    // While we don't access anything in this set twice, unsafe blocks aren't really worth it imo
    let peer_locks = peers.iter_mut()
        .map(|x| (x.0, Mutex::new((x.1, x.2))))
        .collect::<BTreeMap<_,_>>();

    // Task pool for performant, pleasing parallel processing of ports
    rayon::scope(|scope| {
        for (port, socket) in sockets.iter() {
            scope.spawn(|_| {
                let peers = &socket.peers;

                // Take locks for all the peers in our table
                let mut peer_locks = peers.iter()
                .map(|id| {
                    let lock = match peer_locks.get(id).unwrap().try_lock() {
                        Ok(lock) => lock,
                        Err(error) => {
                            panic!("Peer data mutex in receiving system may have had two simultaneous locks, this should not happen. Error is as follows: {error}");
                        },
                    };
                    (lock.1.address, (*id, lock))
                })
                .collect::<BTreeMap<_, _>>();

                // Read all packets
                let mut buffer = [0u8; MAXIMUM_TRANSPORT_UNITS];
                loop {
                    // Read a packet from the socket
                    let (len, origin) = match socket.socket.recv_from(&mut buffer) {
                        Ok(v) => v,
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                        Err(e) => {
                            error!("Error while reading UDP packets: {e}");
                            break
                        },
                    };
                    
                    todo!();
                }
            });
        }
    });
}