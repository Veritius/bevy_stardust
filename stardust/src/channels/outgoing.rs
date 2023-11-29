use bevy::{prelude::*, ecs::system::SystemParam};
use crate::prelude::*;
use super::{id::ChannelMarker, CHANNEL_ENTITY_DELETED_MESSAGE};

/// Systemparam for queuing messages to send in channel `T`.
#[derive(SystemParam)]
pub struct NetworkWriter<'w, 's, C: Channel> {
    query: Query<'w, 's, &'static mut OutgoingMessages, With<ChannelMarker<C>>>
}

impl<'w, 's, C: Channel> NetworkWriter<'w, 's, C> {
    /// Queues a single message for sending.
    pub fn send(&mut self, to: Entity, octets: impl Into<OctetString>) {
        let mut component = self.component_mut();
        Self::send_inner(component.as_mut(), (to, octets.into()))
    }

    /// Queues several messages for sending.
    pub fn send_batch(&mut self, messages: impl Iterator<Item = (Entity, OctetString)>) {
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
    fn send_inner(component: &mut OutgoingMessages, value: (Entity, OctetString)) {
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