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
    pub fn push(&mut self, message: OutgoingNetworkMessage) {
        self.queue.push(message)
    }
}

/// A message queued for sending by transport layers.
pub struct OutgoingNetworkMessage {
    /// The octet string that will be sent.
    pub data: OctetString,
    /// The target for sending.
    pub target: Entity,
}

pub(crate) fn clear_outgoing(
    world: &mut World,
) {
    let mut world = world.cell();
    let registry = world.resource::<ChannelRegistry>();

    for id in registry.channel_ids() {
        let data = registry.get_from_id(id).unwrap();
    }
}