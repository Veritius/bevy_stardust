use std::{sync::Mutex, collections::BTreeMap};
use bevy::prelude::*;
use bevy_stardust::{prelude::*, connections::groups::NetworkGroup};
use crate::{prelude::*, ports::BoundSocketManager};
use super::packing::pack_strings_first_fit;

pub(crate) fn blocking_send_packets_system(
    registry: Res<ChannelRegistry>,
    sockets: Res<BoundSocketManager>,
    groups: Query<&NetworkGroup>,
    mut peers: Query<(Entity, &mut NetworkPeer, &mut UdpConnection)>,
    outgoing: NetworkOutgoingReader,
) {
    // Global values for how we should write packets
    let use_short_ids = registry.channel_count() < u16::MAX.into();

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

                let packed = pack_strings_first_fit(use_short_ids, items);
            })
        }
    });
}