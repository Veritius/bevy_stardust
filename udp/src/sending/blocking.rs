use std::{sync::Mutex, collections::BTreeMap};
use bevy::prelude::*;
use bevy_stardust::{prelude::*, connections::groups::NetworkGroup};
use crate::{prelude::*, ports::BoundSocketManager};
use super::packing::{pack_strings_first_fit, PackingConfig};

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

    // Packing configuration for writing data
    let packing_config = PackingConfig {
        use_short_ids: registry.channel_count() < u16::MAX.into(),
    };

    // Pack data and send to all peers
    rayon::scope(|scope| {
        for (id, mutex) in peer_locks.iter() {
            scope.spawn(|_| {
                let mut peer_data = mutex.try_lock().unwrap();

                let items = outgoing
                .iter_all()
                .filter(|(_, entity, _)| *entity == *id);

                let packed = pack_strings_first_fit(
                    &packing_config,
                    &mut peer_data,
                    items
                );

                todo!();
            })
        }
    });
}