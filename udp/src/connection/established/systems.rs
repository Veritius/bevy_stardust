use bevy_stardust::prelude::*;
use crate::prelude::*;
use super::*;

pub(in crate::connection) fn established_closing_system(
    mut connections: Query<(Entity, &mut Connection, Option<&mut NetworkPeerLifestage>), With<Established>>,
    mut endpoints: Query<&mut Endpoint>,
    mut commands: Commands,
    mut start_events: EventReader<DisconnectPeerEvent>,
    mut fin_events: EventWriter<PeerDisconnectedEvent>,
) {
    for event in start_events.read() {
        if let Ok((_entity, mut connection, _lifestage)) = connections.get_mut(event.peer) {
            connection.closing.begin_local_close(event.reason.clone());
            if event.force { connection.closing.finish_close(); }
        }
    }

    for (entity, connection, lifestage) in connections.iter_mut() {
        if connection.closing.is_closed() {
            info!("Connection {entity:?} closed");
            commands.entity(entity).despawn();

            fin_events.send(PeerDisconnectedEvent {
                peer: entity,
                reason: connection.closing.reason(),
            });

            if let Some(mut lifestage) = lifestage {
                *lifestage = NetworkPeerLifestage::Closed;
            }

            if let Ok(mut endpoint) = endpoints.get_mut(connection.owning_endpoint) {
                endpoint.remove_peer(entity);
            }
        }
    }
}