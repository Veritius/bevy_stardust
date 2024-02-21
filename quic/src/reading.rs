use std::collections::HashMap;
use bevy_ecs::prelude::*;
use bevy_stardust::{prelude::*, channels::registry::ChannelRegistry, connections::{groups::NetworkGroup, peer::NetworkPeer}};
use quinn_proto::Dir;
use crate::{streams::IncomingBufferedStreamData, QuicConnection};

pub(super) fn read_messages_from_streams_system(
    network_groups: Query<&NetworkGroup>,
    mut connections: Query<(Entity, &mut QuicConnection), With<NetworkPeer>>,
    registry: Res<ChannelRegistry>,
    mut reader: NetworkIncomingWriter,
) {
    // Any processing that can run in parallel runs here
    connections.par_iter_mut().for_each(|(entity, mut connection)| {
        // Accept all new streams
        while let Some(stream_id) = connection.inner.get_mut().streams().accept(Dir::Uni) {
            connection.recv_streams.insert(stream_id, IncomingBufferedStreamData::Unverified(Vec::new()));
        }

        // Split borrow function to help out borrowck
        // Mutably borrowing multiple struct fields is not valid apparently
        #[inline]
        fn split_borrow(connection: &mut QuicConnection) -> (
            &mut quinn_proto::Connection,
            &mut HashMap<quinn_proto::StreamId, IncomingBufferedStreamData>,
        ) {(
            connection.inner.get_mut(),
            &mut connection.recv_streams
        )}

        // Borrow many fields using the split borrow function
        let (
            connection_inner,
            active_recv_streams,
        ) = split_borrow(&mut connection);
    });
}