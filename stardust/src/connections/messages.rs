use std::{marker::PhantomData, ops::{Deref, DerefMut}};
use bevy::prelude::*;
use crate::messages::{MessageQueue, NetDirectionType};

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
}

impl<D: NetDirectionType> Deref for PeerMessages<D> {
    type Target = MessageQueue;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: NetDirectionType> DerefMut for PeerMessages<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
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