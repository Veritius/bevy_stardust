use std::{sync::Mutex, collections::BTreeMap};
use bevy::prelude::*;
use bevy_stardust::{prelude::*, connections::groups::NetworkGroup};
use crate::{prelude::*, ports::BoundSocketManager};

pub(crate) fn blocking_send_packets_system(
    registry: Res<ChannelRegistry>,
    sockets: Res<BoundSocketManager>,
    groups: Query<&NetworkGroup>,
    mut peers: Query<(Entity, &mut NetworkPeer, &mut UdpConnection)>,
    outgoing: NetworkOutgoingReader,
) {
    // Mutexes to make the borrow checker happy
    // While we don't access anything in this set twice, unsafe blocks aren't really worth it imo
    let peer_locks = peers.iter_mut()
        .map(|x| (x.0, Mutex::new((x.1, x.2))))
        .collect::<BTreeMap<_,_>>();

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
                            panic!("Peer data mutex in sending system may have had two simultaneous locks, this should not happen. Error is as follows: {error}");
                        },
                    };
                    (*id, lock)
                })
                .collect::<BTreeMap<_, _>>();

                let iter = outgoing
                .iter_all()
                .filter(|(_, target, _)| { peers.binary_search(target).is_ok() });

                // Buffers for a simple algorithm to try and pack as many messages into one packet as possible
                // TODO: This could be made less allocation-heavy
                let mut buffers: BTreeMap<Entity, Vec<Vec<u8>>> = BTreeMap::new();

                // Read all messages and try to pack them into our target's buffers
                for (channel, target, data) in iter {
                    // The target could be a group, so we need to account for that
                    let slice = &[target]; // TODO: This is janky
                    let targets = match groups.get(target) {
                        Ok(v) => v.0.as_slice(),
                        Err(_) => slice,
                    };

                    // Pack the data for all targets
                    for target in targets {
                        let mut buffers = buffers.entry(*target).or_insert(vec![]);
                        todo!()
                    }
                }
            });
        }
    });
}