use std::time::Duration;
use bevy::ecs::entity::EntityHashSet;
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

#[derive(Event)]
pub(in crate::connection) struct DisconnectEstablishedPeerEvent {
    inner: DisconnectPeerEvent,
}

pub(in crate::connection) fn established_close_events_system(
    mut commands: Commands,
    mut transport_events: EventReader<DisconnectEstablishedPeerEvent>,
    mut stardust_events: EventReader<DisconnectPeerEvent>,
    mut connections: Query<(Entity, &mut Established, Option<&mut NetworkPeerLifestage>), With<Connection>>,
) {
    let mut processed_map = EntityHashSet::default();
    let iter = transport_events.read().map(|f| &f.inner).chain(stardust_events.read());
    for event in iter {
        if processed_map.contains(&event.peer) { continue }
        processed_map.insert(event.peer);

        // The error case means that the entity is a network peer we don't control
        // Or, the peer entity was spuriously deleted without going through the right steps
        if let Ok((entity, mut established, lifestage)) = connections.get_mut(event.peer) {
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