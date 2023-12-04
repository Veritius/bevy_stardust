use std::{sync::Mutex, collections::BTreeMap};
use bevy::prelude::*;
use bevy_stardust::{prelude::*, connections::groups::NetworkGroup};
use crate::{prelude::*, ports::BoundSocketManager, MAXIMUM_PACKET_LENGTH};

pub(crate) fn blocking_send_packets_system(
    registry: Res<ChannelRegistry>,
    sockets: Res<BoundSocketManager>,
    groups: Query<&NetworkGroup>,
    mut peers: Query<(Entity, &mut NetworkPeer, &mut UdpConnection)>,
    outgoing: NetworkOutgoingReader,
) {
    // Mutexes to make the borrow checker happy
    let peer_locks = peers.iter_mut()
        .map(|x| (x.0, Mutex::new((x.1, x.2))))
        .collect::<BTreeMap<_,_>>();

    // Step 1: Create the packets that need sending to each peer
    rayon::scope(|scope| {
        for (id, mutex) in peer_locks.iter() {
            scope.spawn(|_| {
                let mut lock = mutex.try_lock().unwrap();

                let items = outgoing
                .iter_all()
                .filter(|(_, entity, _)| *entity == *id);

                // Buffers for a simple algorithm to try and pack as many octet strings into one packet as possible
                // The algorithm just finds the first 'buffer' and writes the data to that, adding more buffers as necessary.
                let mut buffers: Vec<Vec<u8>> = vec![];

                // Scratch space
                let mut scratch_buf = [0u8; 1450];
                let mut scratch_len: usize = 0;

                // Pack all strings into packets
                for (channel, _, string) in items {
                    // Check the string isn't too long since fragmenting isn't supported
                    if string.len() > (MAXIMUM_PACKET_LENGTH - 20) {
                        panic!("A sent octet string was too long ({} bytes). Fragmenting isn't supported right now, so it couldn't be sent.", string.len());
                    }

                    todo!()
                }
            })
        }
    });
}