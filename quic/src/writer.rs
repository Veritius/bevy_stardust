use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::connection::QuicConnection;

pub(super) fn quic_writer_system(
    mut connections: Query<(&mut NetworkPeer, &mut QuicConnection)>,
    messages: NetworkOutgoingReader,
) {
    for (channel, origin, bytes) in messages.iter_all() {
        
    }
}