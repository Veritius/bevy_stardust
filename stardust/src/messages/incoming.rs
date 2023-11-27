//! Incoming message events.

use std::marker::PhantomData;
use bevy::prelude::*;
use crate::prelude::{Channel, OctetString};

/// An octet string that has been received by a transport layer.
#[derive(Event)]
pub struct NetworkMessage<T: Channel> {
    /// The bytes sent over the network.
    pub data: OctetString,
    /// The peer this message was received from.
    pub origin: Entity,
    phantom: PhantomData<T>,
}

impl<T: Channel> NetworkMessage<T> {
    /// Creates a new `NetworkMessage<T>` event.
    pub fn new(origin: Entity, data: impl Into<OctetString>) -> Self {
        Self {
            data: data.into(),
            origin,
            phantom: PhantomData,
        }
    }
}