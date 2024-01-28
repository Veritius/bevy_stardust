use std::{collections::BTreeMap, sync::Mutex};
use bevy::prelude::*;
use bevy_stardust::connections::peer::NetworkPeer;
use crate::{sockets::SocketManager, connection::UdpConnection};

pub(crate) fn packet_sender_system(
    mut connections: Query<(&mut NetworkPeer, &mut UdpConnection)>,
    mut sockets: ResMut<SocketManager>,
) {
    // Accessing client data through a mutex is probably fine
    let connections = connections
        .iter_mut()
        .map(|(a,b)| {
            (b.address().clone(), Mutex::new((a, b)))
        })
        .collect::<BTreeMap<_, _>>();

    // Explicit borrows to prevent moves
    let connections = &connections;

    // Iterate over all sockets
    rayon::scope(|scope| {
        for socket in sockets.iter_sockets() {
            scope.spawn(move |_| {
                todo!();
            });
        }
    });
}