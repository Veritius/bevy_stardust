use bevy::prelude::*;
use crate::prelude::*;

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