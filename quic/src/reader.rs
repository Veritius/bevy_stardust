use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::connection::QuicConnection;

pub(super) fn quic_reader_system(
    mut connections: Query<(&mut NetworkPeer, &mut QuicConnection)>,
    mut messages: NetworkIncomingWriter,
) {
    for (peer_data, mut connection) in connections.iter_mut() {

    }
}