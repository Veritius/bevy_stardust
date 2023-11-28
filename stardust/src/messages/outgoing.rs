//! Outgoing message queue.

use std::marker::PhantomData;
use bevy::prelude::*;
use crate::prelude::{Channel, OctetString, ChannelRegistry};

/// Messages on channel `T` that are queued for sending.
#[derive(Resource)]
pub struct OutgoingNetworkMessages<T: Channel> {
    pub(crate) queue: Vec<OutgoingNetworkMessage>,
    phantom: PhantomData<T>,
}

impl<T: Channel> Default for OutgoingNetworkMessages<T> {
    fn default() -> Self {
        Self {
            queue: Default::default(),
            phantom: Default::default()
        }
    }
}

impl<T: Channel> OutgoingNetworkMessages<T> {
    /// Pushes `message` to the queue.
    pub fn push(&mut self, message: impl Into<OutgoingNetworkMessage>) {
        self.queue.push(message.into())
    }
}

/// A message queued for sending by transport layers.
pub struct OutgoingNetworkMessage {
    /// The octet string that will be sent.
    pub data: OctetString,
    /// The target for sending.
    pub target: Entity,
}

impl From<(OctetString, Entity)> for OutgoingNetworkMessage {
    fn from(value: (OctetString, Entity)) -> Self {
        Self {
            data: value.0,
            target: value.1,
        }
    }
}

impl From<(Entity, OctetString)> for OutgoingNetworkMessage {
    fn from(value: (Entity, OctetString)) -> Self {
        Self {
            data: value.1,
            target: value.0,
        }
    }
}

pub(crate) fn clear_outgoing(
    world: &mut World,
) {
    todo!()
}