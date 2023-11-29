use bevy::{prelude::*, ecs::system::SystemParam};
use crate::prelude::*;
use super::{id::ChannelMarker, CHANNEL_ENTITY_DELETED_MESSAGE};

/// Systemparam for reading messages received in channel `T`.
#[derive(SystemParam)]
pub struct NetworkReader<'w, 's, C: Channel> {
    query: Query<'w, 's, &'static IncomingMessages, With<ChannelMarker<C>>>
}

impl<'w, 's, C: Channel> NetworkReader<'w, 's, C> {
    /// Returns an iterator over all messages in this channel.
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

/// Incoming messages on this channel.
#[derive(Component)]
pub(super) struct IncomingMessages {
    pub queue: Vec<(Entity, OctetString)>,
}

pub(super) fn clear_incoming(
    mut query: Query<&mut IncomingMessages>,
) {
    query.par_iter_mut()
    .for_each(|mut v| v.queue.clear());
}