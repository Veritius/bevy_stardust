use std::marker::PhantomData;
use bevy::prelude::*;
use crate::{channels::ChannelId, messages::*};
use super::Peer;

/// A message queue for a [peer entity], exposing a subset of [`MessageQueue`]'s API.
/// 
/// [`PeerMessages<D>`] has a generic `D`, which defines its [direction].
/// 
/// [`PeerMessages<D>`] components are cleared automatically in the [`NetworkSend::Clear`] system set.
/// Unread messages will be discarded unless the [`Message`] objects are cloned.
/// 
/// [peer entity]: crate::connections
/// [direction]: crate::messages::NetDirection
/// [`NetworkSend::Clear`]: crate::scheduling::NetworkSend::Clear
#[derive(Default, Component)]
pub struct PeerMessages<D: MessageDirection> {
    queue: MessageQueue,
    phantom: PhantomData<D>,
}

impl<D: MessageDirection> PeerMessages<D> {
    /// Creates a new [`PeerMessages<D>`].
    pub fn new() -> Self {
        Self {
            queue: MessageQueue::new(),
            phantom: PhantomData,
        }
    }

    /// Returns the total number of messages stored in the queue.
    #[inline]
    pub fn count(&self) -> usize {
        self.queue.count()
    }

    /// Returns the sum of bytes from all messages in the queue.
    #[inline]
    pub fn bytes(&self) -> usize {
        self.queue.bytes()
    }

    /// Pushes a single message to the queue.
    #[inline]
    pub fn push_one(&mut self, message: ChannelMessage) {
        self.queue.push_one(message);
    }

    /// Pushes many messages from `iter` to the queue.
    /// This can be faster than calling [`push_one`](Self::push_one) repeatedly.
    #[inline]
    pub fn push_many<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = ChannelMessage>,
    {
        self.queue.push_many(iter);
    }

    /// Pushes many messages from `iter` to a single channel.
    /// This can be faster than calling [`push_one`](Self::push_one) or [`push_many`](Self::push_many) repeatedly.
    #[inline]
    pub fn push_channel<I>(&mut self, channel: ChannelId, iter: I)
    where
        I: IntoIterator<Item = Message>,
    {
        self.queue.push_channel(channel, iter);
    }

    /// Returns an iterator over channels, and their associated queues.
    #[inline]
    pub fn iter(&self) -> ChannelIter {
        self.queue.iter()
    }

    /// Returns an iterator over all messages in a specific channel.
    #[inline]
    pub fn iter_channel(&self, channel: ChannelId) -> MessageIter {
        self.queue.iter_channel(channel)
    }
}

impl<'a, D: MessageDirection> IntoIterator for &'a PeerMessages<D> {
    type Item = <&'a MessageQueue as IntoIterator>::Item;
    type IntoIter = <&'a MessageQueue as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.queue.into_iter()
    }
}

impl<D: MessageDirection> AsRef<MessageQueue> for PeerMessages<D> {
    /// Borrows the inner [`MessageQueue`].
    #[inline]
    fn as_ref(&self) -> &MessageQueue {
        &self.queue
    }
}

impl<D: MessageDirection> AsMut<MessageQueue> for PeerMessages<D> {
    /// Mutably borrows the inner [`MessageQueue`].
    #[inline]
    fn as_mut(&mut self) -> &mut MessageQueue {
        &mut self.queue
    }
}

pub(crate) fn clear_message_queues_system<D: MessageDirection>(
    mut instances: Query<&mut PeerMessages<D>, With<Peer>>,
) {
    for mut messages in instances.iter_mut() {
        messages.queue.clear()
    }
}