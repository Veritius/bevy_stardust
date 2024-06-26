use std::{marker::PhantomData, ops::Deref};
use bevy::prelude::*;
use crate::messages::{MessageQueue, NetDirectionType, ChannelMessage, Message, ChannelId};

/// A message queue for a [peer entity].
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
}

impl<D: NetDirectionType> Deref for PeerMessages<D> {
    type Target = MessageQueue;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
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