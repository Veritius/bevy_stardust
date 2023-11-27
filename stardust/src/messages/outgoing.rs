//! Outgoing message queue.

use std::marker::PhantomData;
use bevy::prelude::*;
use crate::prelude::{Channel, OctetString};

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

pub struct OutgoingNetworkMessage {
    pub data: OctetString,
    pub target: Entity,
}

pub(crate) fn clear_outgoing(
    
) {
    todo!()
}