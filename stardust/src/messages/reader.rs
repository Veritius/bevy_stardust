//! Systemparams for game systems to read messages.

use std::marker::PhantomData;
use bevy::{prelude::*, ecs::system::SystemParam};
use crate::prelude::{Channel, OctetString, ChannelRegistry};
use super::incoming::IncomingMessageQueue;

/// Allows game systems to read messages sent over channel `T`.
#[derive(SystemParam)]
pub struct MessageReader<'w, 's, T: Channel> {
    registry: Res<'w, ChannelRegistry>,
    storages: Query<'w, 's, &'static IncomingMessageQueue>,
    phantom: PhantomData<T>,
}

impl<'w, 's, T: Channel> MessageReader<'w, 's, T> {
    /// Reads all messages from peer `from` on channel `T`.
    pub fn read_from(&self, from: Entity) -> Option<&[OctetString]> {
        match self.storages.get(from).ok() {
            Some(val) => {
                let (cid, _) = self.registry.get_from_type::<T>()
                    .expect("Tried to access messages from a channel that wasn't registered");
                Some(val.read_channel(cid))
            },
            None => None,
        }
    }
}