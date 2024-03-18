use bevy_ecs::prelude::*;
use super::{NetworkPeer, NetworkPeerLifestage};

pub(crate) fn despawn_closed_connections_system(
    mut commands: Commands,
    query: Query<(Entity, &NetworkPeerLifestage), (With<NetworkPeer>, Changed<NetworkPeerLifestage>)>,
) {
    for (id, stage) in query.iter() {
        if *stage == NetworkPeerLifestage::Closed {
            commands.entity(id).despawn();
        }
    }
}