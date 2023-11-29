use bevy::{prelude::*, ecs::system::SystemParam};
use crate::prelude::*;
use super::id::ChannelMarker;

/// Systemparam for queuing messages to send in channel `T`.
#[derive(SystemParam)]
pub struct NetworkWriter<'w, 's, T: Channel> {
    query: Query<'w, 's, &'static mut OutgoingMessages, With<ChannelMarker<T>>>
}

/// Systemparam for reading all messages queued to be sent this frame.
/// This is intended for use in transport layers.
#[derive(SystemParam)]
pub struct NetworkOutgoingReader<'w, 's> {
    registry: Res<'w, ChannelRegistry>,
    query: Query<'w, 's, &'static OutgoingMessages>
}

/// Queued outgoing messages on this channel.
#[derive(Component)]
pub(super) struct OutgoingMessages {
    pub queue: Vec<(Entity, OctetString)>,
}

pub(super) fn clear_outgoing(
    mut query: Query<&mut OutgoingMessages>,
) {
    query.par_iter_mut()
    .for_each(|mut v| v.queue.clear());
}