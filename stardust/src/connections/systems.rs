use bevy::prelude::*;
use super::{Peer, PeerLifestage};

pub(crate) fn despawn_closed_connections_system(
    mut commands: Commands,
    query: Query<(Entity, &PeerLifestage), (With<Peer>, Changed<PeerLifestage>)>,
) {
    for (id, stage) in query.iter() {
        if *stage == PeerLifestage::Closed {
            commands.entity(id).despawn();
        }
    }
}