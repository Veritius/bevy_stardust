use bevy_ecs::prelude::*;
use crate::{Connection, ConnectionState, Endpoint};

// TODO: Use change detection.
pub(crate) fn close_connections_system(
    mut commands: Commands,
    mut endpoints: Query<&mut Endpoint>,
    connections: Query<(Entity, &Connection)>,
) {
    // This doesn't need to be in parallel.
    for (entity, connection) in connections.iter() {
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