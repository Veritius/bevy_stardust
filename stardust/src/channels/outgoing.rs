use bevy::{prelude::*, ecs::system::SystemParam};
use bytes::Bytes;
use crate::prelude::*;
use super::{id::ChannelMarker, CHANNEL_ENTITY_DELETED_MESSAGE};

/// Systemparam for queuing messages to send in channel `C`.
/// 
/// ## Panics
/// Using any of the methods in this systemparam will panic if `C` wasn't registered in the `App`.
/// In future, this may change to panicking the moment the scheduler tries to run the system.
#[derive(SystemParam)]
pub struct NetworkWriter<'w, 's, C: Channel> {
    query: Query<'w, 's, &'static mut OutgoingMessages, With<ChannelMarker<C>>>
}

impl<'w, 's, C: Channel> NetworkWriter<'w, 's, C> {
    /// Queues a single message for sending.
    pub fn send(&mut self, to: Entity, octets: Bytes) {
        let mut component = self.component_mut();
        Self::send_inner(component.as_mut(), (to, octets.into()))
    }

    /// Queues several messages for sending.
    pub fn send_many(&mut self, messages: impl Iterator<Item = (Entity, Bytes)>) {
        let mut component = self.component_mut();
        for value in messages {
            Self::send_inner(component.as_mut(), value);
        }
    }

    /// Returns how many messages have been queued for sending so far.
    pub fn count(&self) -> usize {
        self.component().queue.len()
    }

    #[inline]
    fn component(&self) -> &OutgoingMessages {
        self.query.get_single().expect(CHANNEL_ENTITY_DELETED_MESSAGE)
    }

    #[inline]
    fn component_mut(&mut self) -> Mut<'_, OutgoingMessages> {
        self.query.get_single_mut().expect(CHANNEL_ENTITY_DELETED_MESSAGE)
    }

    #[inline]
    fn send_inner(component: &mut OutgoingMessages, value: (Entity, Bytes)) {
        component.queue.push(value);
    }
}

/// Systemparam for reading all messages queued to be sent this frame.
/// This is intended for use in transport layers.
#[derive(SystemParam)]
pub struct NetworkOutgoingReader<'w, 's> {
    registry: Res<'w, ChannelRegistry>,
    query: Query<'w, 's, &'static OutgoingMessages>
}

impl<'w, 's> NetworkOutgoingReader<'w, 's> {
    // what a clusterfuck of a function signature lol
    /// Returns an iterator that can be used to read all outgoing messages for each channel.
    /// 
    /// ```ignore
    /// // Example usage
    /// // Outer iterator reads over channels
    /// for (channel, msg_iter) in reader.iter_channels() {
    ///     // Inner iterator reads over octet strings in those channels
    ///     for (origin, string) in msg_iter {
    ///         println!("{origin:?} sent {string:?} on channel {channel:?}");
    ///     }
    /// }
    /// ```
    pub fn iter_channels(&self) -> impl Iterator<Item = (ChannelId, impl Iterator<Item = &(Entity, Bytes)>)> {
        self.registry.channel_ids()
        .map(|id| {
            let data = self.registry.get_from_id(id).unwrap();
            let (id, entity) = (data.channel_id, data.entity_id);
            let component = self.query.get(entity)
                .expect(CHANNEL_ENTITY_DELETED_MESSAGE);
            (id, component.queue.iter())
        })
    }

    /// Returns an iterator over all messages and their channel and sender data.
    /// 
    /// ```ignore
    /// // Example usage
    /// for (channel, origin, string) in reader.iter_all() {
    ///     println!("{origin:?} sent {string:?} on channel {channel:?}");
    /// }
    /// ```
    pub fn iter_all(&self) -> impl Iterator<Item = (ChannelId, Entity, &Bytes)> {
        self.iter_channels()
        .flat_map(|f| {
            f.1.map(move |x| (f.0, x.0, &x.1))
        })
    }
}

/// Queued outgoing messages on this channel.
#[derive(Default, Component)]
pub(super) struct OutgoingMessages {
    pub queue: Vec<(Entity, Bytes)>,
}

pub(super) fn clear_outgoing(
    mut query: Query<&mut OutgoingMessages>,
) {
    query.par_iter_mut()
    .for_each(|mut v| v.queue.clear());
}