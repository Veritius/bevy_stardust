use std::{collections::BTreeMap, sync::Mutex};
use bevy::prelude::*;
use bevy_stardust::connections::peer::NetworkPeer;
use crate::{sockets::SocketManager, connection::UdpConnection};

pub(crate) fn packet_sender_system(
    mut connections: Query<(Entity, &mut NetworkPeer, &mut UdpConnection)>,
    mut sockets: ResMut<SocketManager>,
) {
    let addresses = connections
        .iter()
        .map(|(a, _, b)| {
            (b.address().clone(), a)
        })
        .collect::<BTreeMap<_, _>>();

    let clients = connections
        .iter_mut()
        .map(|(a,b,c)| {
            (a, Mutex::new((b,c)))
        })
        .collect::<BTreeMap<_, _>>();

    // Iterate over all sockets
    rayon::scope(|scope| {
        for socket in sockets.iter_sockets() {
            scope.spawn(|_| {
                todo!();
            });
        }
    });
}