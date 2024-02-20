use bevy_ecs::prelude::*;
use bevy_stardust::{prelude::*, channels::registry::ChannelRegistry, connections::{groups::NetworkGroup, peer::NetworkPeer}};
use quinn_proto::Dir;
use crate::QuicConnection;

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
            connection.pending_recv_streams.push((stream_id, Vec::new()));
        }

        todo!()
    });
}