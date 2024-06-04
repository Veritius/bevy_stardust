use std::time::Duration;
use bevy_stardust::prelude::*;
use crate::prelude::*;
use super::*;

#[derive(Debug, Component)]
pub(in crate::connection) struct Closing {
    finished: bool,
    informed: bool,
    origin: CloseOrigin,
    reason: Option<Bytes>,
    timeout: Duration,
}

impl Closing {
    pub(super) fn new(
        origin: CloseOrigin,
        reason: Option<Bytes>,
        timeout: Duration,
    ) -> Self {
        Self {
            finished: false,
            informed: false,
            origin,
            reason,
            timeout,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CloseOrigin {
    Local,
    Remote,
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

            commands.entity(entity).insert(Closing::new(
                CloseOrigin::Local,
                event.reason.clone(),
                Duration::from_secs(5),
            ));
        }
    }
}

pub(in crate::connection) fn established_close_frames_system(
    mut connections: Query<(Entity, &mut Established, &mut Closing)>,
) {

}

pub(in crate::connection) fn established_close_despawn_system(

) {

}