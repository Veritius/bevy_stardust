use std::{collections::HashMap, marker::PhantomData};
use bevy::prelude::*;
use bytes::Bytes;
use crate::prelude::*;
use super::direction::DirectionType;

static EMPTY_SLICE: &[Bytes] = &[];

/// A queue-like structure for storing messages, separated by channels.
/// 
/// The items in this queue **do not** persist across frames.
/// They are cleared in [`NetworkWrite::Clear`] in [`PostUpdate`].
#[derive(Component, Reflect)]
#[reflect(Debug, Component)]
pub struct NetworkMessages<D: DirectionType> {
    #[reflect(ignore)]
    pub(crate) queue_map: HashMap<ChannelId, Vec<Bytes>>,
    #[reflect(ignore)]
    phantom: PhantomData<D>
}

impl<D: DirectionType> NetworkMessages<D> {
    /// Creates a new `Messages` store. Doesn't allocate until [`push`](Self::push) is used.
    pub fn new() -> Self {
        Self {
            queue_map: HashMap::new(),
            phantom: PhantomData,
        }
    }

    /// Clears all queues but doesn't reallocate.
    pub(crate) fn clear(&mut self) {
        self.queue_map
        .iter_mut()
        .for_each(|(_, v)| v.clear())
    }

    /// Counts how many messages are queued in all channels.
    pub fn count(&self) -> usize {
        self.queue_map
        .iter()
        .map(|(_,v)| v.len())
        .sum()
    }

    /// Pushes a new item to the queue.
    pub fn push(&mut self, channel: ChannelId, bytes: Bytes) {
        self.queue_map
        .entry(channel)
        .or_insert(Vec::with_capacity(1))
        .push(bytes);
    }

    /// Returns a slice of the queue for channel `channel`.
    pub fn channel_queue(&self, channel: ChannelId) -> &[Bytes] {
        self.queue_map
        .get(&channel)
        .map_or(EMPTY_SLICE, |v| v.as_slice())
    }

    /// Returns an iterator over all queues, including their channel ids.
    /// The iterator does not contain empty queues.
    pub fn all_queues(&self) -> impl Iterator<Item = (ChannelId, &[Bytes])> {
        self.queue_map
        .iter()
        .filter(|(_,v)| v.len() != 0)
        .map(|(k,v)| (k.clone(), v.as_slice()))
    }
}

impl<D: DirectionType> Default for NetworkMessages<D> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<D: DirectionType> std::fmt::Debug for NetworkMessages<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("NetworkMessages<{}>", std::any::type_name::<D>()))
    }
}