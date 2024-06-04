use bevy_stardust::prelude::*;
use crate::prelude::*;
use super::*;

#[derive(Debug)]
pub(in super) struct Closing {
    finished: bool,
    informed: bool,
    origin: CloseOrigin,
    reason: Option<Bytes>,
}

impl Closing {
    pub fn new(
        origin: CloseOrigin,
        reason: Option<Bytes>,
    ) -> Self {
        Self {
            finished: false,
            informed: false,
            origin,
            reason,
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
            if established.as_ref().closing.is_some() { continue } // Already closing
            established.closing = Some(Closing::new(
                CloseOrigin::Local,
                event.reason.clone(),
            ));
        }
    }
}