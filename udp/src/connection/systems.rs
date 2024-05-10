use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::prelude::*;
use super::ticking::*;

pub(crate) fn connection_preupdate_ticking_system(
    registry: Res<ChannelRegistry>,
    mut connections: Query<(&mut Connection, Option<&mut NetworkMessages<Incoming>>)>,
) {
    // Tick all connections in parallel
    connections.par_iter_mut().for_each(|(mut connection, messages)| {
        connection.inner_mut().tick_preupdate(PreUpdateTickData {
            registry: &registry,
            messages,
        });
    });
}

pub(crate) fn connection_postupdate_ticking_system(
    registry: Res<ChannelRegistry>,
    mut connections: Query<(&mut Connection, Option<Ref<NetworkMessages<Outgoing>>>)>,
) {
    // Tick all connections in parallel
    connections.par_iter_mut().for_each(|(mut connection, messages)| {
        connection.inner_mut().tick_postupdate(PostUpdateTickData {
            registry: &registry,
            messages,
        });
    });
}

// TODO: Use change detection.
pub(crate) fn close_connections_system(
    mut commands: Commands,
    mut endpoints: Query<&mut Endpoint>,
    connections: Query<(Entity, &Connection), Changed<Connection>>,
) {
    // This doesn't need to be in parallel.
    for (entity, connection) in connections.iter() {
        let connection = connection.inner();
        if connection.state() == ConnectionState::Closed {
            // Despawn entity
            commands.entity(entity).despawn();

            // Remove from the connection map
            if let Ok(mut endpoint) = endpoints.get_mut(connection.owning_endpoint) {
                endpoint.remove_peer(entity);
            }
        }
    }
}