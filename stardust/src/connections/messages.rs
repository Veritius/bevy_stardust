use std::marker::PhantomData;
use bevy::prelude::*;
use crate::messages::{ChannelId, ChannelIter, ChannelMessage, Message, MessageIter, MessageQueue, NetDirectionType};
use super::Peer;

/// A message queue for a [peer entity], exposing a subset of [`MessageQueue`]'s API.
/// The generic `D` defines its [direction].
/// 
/// [peer entity]: crate::connections
/// [direction]: crate::messages::NetDirection
#[derive(Component)]
pub struct PeerMessages<D: NetDirectionType> {
    inner: MessageQueue,
    phantom: PhantomData<D>,
}

impl<D: NetDirectionType> PeerMessages<D> {
    /// Creates a new [`PeerMessages<D>`].
    pub fn new() -> Self {
        Self {
            inner: MessageQueue::new(),
            phantom: PhantomData,
        }
    }

    /// Returns the total number of messages stored in the queue.
    #[inline]
    pub fn count(&self) -> usize {
        self.inner.count()
    }

    /// Returns the sum of bytes from all messages in the queue.
    #[inline]
    pub fn bytes(&self) -> usize {
        self.inner.bytes()
    }

    /// Pushes a single message to the queue.
    #[inline]
    pub fn push_one(&mut self, message: ChannelMessage) {
        self.inner.push_one(message);
    }

    /// Pushes many messages from `iter` to the queue.
    /// This can be faster than calling [`push_one`](Self::push_one) repeatedly.
    #[inline]
    pub fn push_many<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = ChannelMessage>,
    {
        self.inner.push_many(iter);
    }

    /// Pushes many messages from `iter` to a single channel.
    /// This can be faster than calling [`push_one`](Self::push_one) or [`push_many`](Self::push_many) repeatedly.
    #[inline]
    pub fn push_channel<I>(&mut self, channel: ChannelId, iter: I)
    where
        I: IntoIterator<Item = Message>,
    {
        self.inner.push_channel(channel, iter);
    }

    /// Returns an iterator over channels, and their associated queues.
    #[inline]
    pub fn iter(&self) -> ChannelIter {
        self.inner.iter()
    }

    /// Returns an iterator over all messages in a specific channel.
    #[inline]
    pub fn iter_channel(&self, channel: ChannelId) -> MessageIter {
        self.inner.iter_channel(channel)
    }
}

impl<'a, D: NetDirectionType> IntoIterator for &'a PeerMessages<D> {
    type Item = <&'a MessageQueue as IntoIterator>::Item;
    type IntoIter = <&'a MessageQueue as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

pub(crate) fn clear_message_queues_system<D: NetDirectionType>(
    mut instances: Query<&mut PeerMessages<D>, With<Peer>>,
) {
    for mut messages in instances.iter_mut() {
        messages.inner.clear()
    }
}