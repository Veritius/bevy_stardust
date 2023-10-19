//! Systemparams for game systems to write messages.

use std::marker::PhantomData;
use bevy::{prelude::*, ecs::system::SystemParam};
use crate::prelude::{Channel, OctetString};
use super::outgoing::{ChannelOctetStringCollectionArcHolder, SendTarget};

/// Allows game systems to send messages to peers on channel `T`.
#[derive(SystemParam)]
pub struct MessageWriter<'w, 's, T: Channel> {
    outgoing: ResMut<'w, ChannelOctetStringCollectionArcHolder<T>>,
    phantom: PhantomData<&'s ()>,
}

impl<'w, 's, T: Channel> MessageWriter<'w, 's, T> {
    /// Queues a message for sending by transport layer.
    pub fn write(&mut self, to: SendTarget, octets: impl Into<OctetString>) {
        let mut write_guard = self.outgoing.internal.write().unwrap();
        write_guard.0.push((to, octets.into()));
    }

    /// Queues several messages for sending by the transport layer.
    /// 
    /// Slightly more efficient than `write` for multiple messages.
    pub fn write_many(&mut self, iter: impl Iterator<Item = (SendTarget, OctetString)>) {
        let mut write_guard = self.outgoing.internal.write().unwrap();
        for item in iter {
            write_guard.0.push(item)
        }
    }
}