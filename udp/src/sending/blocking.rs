use std::{sync::Mutex, collections::BTreeMap};
use bevy::prelude::*;
use bevy_stardust::{prelude::*, connections::groups::NetworkGroup};
use crate::{prelude::*, ports::BoundSocketManager};
use super::assembler::*;

pub(crate) fn blocking_send_packets_system(
    config: Res<PluginConfig>,
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

    // Pack data and send to all peers
    rayon::scope(|scope| {
        for (id, mutex) in peer_locks.iter() {
            scope.spawn(|_| {
                let mut peer_data = mutex.try_lock().unwrap();

                let strings = outgoing
                .iter_all()
                .filter(|(_, entity, _)| *entity == *id)
                .map(|(a, _, b)| (a,b));

                let packets = assemble_packets(
                    &config,
                    &registry,
                    &mut peer_data.1,
                    strings
                );

                todo!();
            })
        }
    });
}