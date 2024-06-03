use bevy_stardust::prelude::*;
use crate::prelude::*;
use super::*;

pub(in crate::connection) fn established_closing_system(
    mut connections: Query<(Entity, &Connection, &Established, Option<&mut NetworkPeerLifestage>)>,
    mut endpoints: Query<&mut Endpoint>,
    mut commands: Commands,
) {
    for (entity, connection, established, lifestage) in connections.iter_mut() {
        match &established.state {
            EstablishedState::Closed => {
                info!("Connection {entity:?} closed");
                commands.entity(entity).despawn();

                if let Some(mut lifestage) = lifestage {
                    *lifestage = NetworkPeerLifestage::Closed;
                }

                if let Ok(mut endpoint) = endpoints.get_mut(connection.owning_endpoint) {
                    endpoint.remove_peer(entity);
                }
            },
            _ => {},
        }
    }
}