use bevy::{prelude::*, ecs::system::SystemParam};
use crate::prelude::*;
use super::id::ChannelMarker;

/// Systemparam for reading messages received in channel `T`.
#[derive(SystemParam)]
pub struct NetworkReader<'w, 's, T: Channel> {
    query: Query<'w, 's, &'static IncomingMessages, With<ChannelMarker<T>>>
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