use bevy_stardust::prelude::*;
use crate::prelude::*;
use super::*;

#[derive(Component)]
pub(super) struct Closing {

}

impl Closing {
    pub fn new() -> Self {
        Self {

        }
    }
}

pub(in crate::connection) fn established_close_events_system(
    mut commands: Commands,
    mut events: EventReader<DisconnectPeerEvent>,
    mut connections: Query<(Entity, &mut Established, Option<&mut NetworkPeerLifestage>), With<Connection>>,
) {
    for event in events.read() {
        // The error case means that the entity is a network peer we don't control
        if let Ok((entity, mut established, lifestage)) = connections.get_mut(event.peer) {
            if established.as_ref().closing { continue } // Already closing
            established.closing = true;
            commands.entity(entity).insert(Closing::new());
        }
    }
}

pub(in crate::connection) fn established_close_frames_system(

) {

}

pub(in crate::connection) fn established_close_despawn_system(

) {

}