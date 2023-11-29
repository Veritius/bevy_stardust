use bevy::{prelude::*, ecs::system::SystemParam};
use crate::prelude::*;
use super::{id::ChannelMarker, CHANNEL_ENTITY_DELETED_MESSAGE};

/// Systemparam for reading messages received in channel `T`.
#[derive(SystemParam)]
pub struct NetworkReader<'w, 's, C: Channel> {
    query: Query<'w, 's, &'static IncomingMessages, With<ChannelMarker<C>>>
}

impl<'w, 's, C: Channel> NetworkReader<'w, 's, C> {
    /// Returns an iterator over all messages in this channel, including the sender's ID.
    pub fn iter(&'w self) -> impl Iterator<Item = &'w (Entity, OctetString)> {
        self.query.get_single().expect(CHANNEL_ENTITY_DELETED_MESSAGE).queue.iter()
    }
}

/// Systemparam for storing messages for reading by `NetworkReader` params.
/// This is intended for use in transport layers.
#[derive(SystemParam)]
pub struct NetworkIncomingWriter<'w, 's> {
    registry: Res<'w, ChannelRegistry>,
    query: Query<'w, 's, &'static mut IncomingMessages>
}

impl<'w, 's> NetworkIncomingWriter<'w, 's> {
    /// Queues a single octet string for reading by the `NetworkReader` corresponding to `channel`.
    pub fn send(&mut self, channel: ChannelId, origin: Entity, string: impl Into<OctetString>) {
        self.get_mut(channel).queue.push((origin, string.into()));
    }

    /// Queues several messages for reading by the `NetworkReader` corresponding to `channel`.
    pub fn send_many(&mut self, channel: ChannelId, messages: impl Iterator<Item = (Entity, impl Into<OctetString>)>) {
        let mut data = self.get_mut(channel);
        for (origin, string) in messages {
            data.queue.push((origin, string.into()));
        }
    }

    #[inline]
    fn get_mut(&mut self, id: ChannelId) -> Mut<'_, IncomingMessages> {
        let data = self.registry.get_from_id(id).unwrap();
        self.query.get_mut(data.entity_id).expect(CHANNEL_ENTITY_DELETED_MESSAGE)
    }
}

/// Incoming messages on this channel.
#[derive(Default, Component)]
pub(super) struct IncomingMessages {
    pub queue: Vec<(Entity, OctetString)>,
}

pub(super) fn clear_incoming(
    mut query: Query<&mut IncomingMessages>,
) {
    query.par_iter_mut()
    .for_each(|mut v| v.queue.clear());
}