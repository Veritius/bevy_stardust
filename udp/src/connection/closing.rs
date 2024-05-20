use std::time::{Duration, Instant};
use bevy::prelude::*;
use bevy_stardust::connections::PeerDisconnectedEvent;
use bytes::Bytes;
use crate::prelude::Endpoint;
use super::Connection;

#[derive(Component)]
pub(crate) struct Closing {
    started: Instant,
    timeout: Duration,

    reason: Option<Bytes>,
    inform: bool,

    finished: bool,
    informed: bool,
}

impl Closing {
    pub fn new(
        reason: Option<Bytes>,
        inform: bool,
    ) -> Self {
        Self {
            started: Instant::now(),
            timeout: Duration::from_secs(10),

            reason,
            inform,

            finished: false,
            informed: false,
        }
    }

    pub fn set_informed(&mut self) {
        self.informed = true;
    }

    pub fn needs_inform(&self) -> bool {
        !self.inform | self.informed
    }

    pub fn set_finished(&mut self) {
        self.finished = true;
    }
}

pub(crate) fn closing_component_system(
    mut commands: Commands,
    mut events: EventWriter<PeerDisconnectedEvent>,
    mut closing: Query<(Entity, &Connection, &mut Closing)>,
    mut endpoints: Query<&mut Endpoint>,
) {
    for (entity, connnection, mut closing) in closing.iter_mut() {
        if closing.started.elapsed() >= closing.timeout {
            closing.finished = true;
        }

        if closing.finished {
            if closing.inform && !closing.informed {
                error!("{entity:?} didn't inform remote peer before closing");
            }

            commands.entity(entity).despawn();
            events.send(PeerDisconnectedEvent {
                peer: entity,
                reason: closing.reason.clone(),
            });

            let mut endpoint = match endpoints.get_mut(connnection.owning_endpoint) {
                Ok(e) => e,
                Err(_) => { todo!() },
            };

            endpoint.connections.remove(&connnection.remote_address);
        }
    }
}