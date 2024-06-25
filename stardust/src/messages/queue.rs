use std::{collections::HashMap, marker::PhantomData};
use bevy::prelude::*;
use bytes::Bytes;
use smallvec::SmallVec;
use crate::prelude::*;
use super::direction::NetDirectionType;

type IdxVec = SmallVec<[usize; 1]>; 

/// A queue-like structure for storing messages, separated by channels.
/// 
/// The items in this queue **do not** persist across frames.
/// They are cleared in [`NetworkWrite::Clear`] in [`PostUpdate`].
#[derive(Component)]
pub struct NetworkMessages<D: NetDirectionType> {
    pub(crate) messages: Vec<Bytes>,
    pub(crate) queue_map: HashMap<ChannelId, IdxVec>,
    phantom: PhantomData<D>
}

impl<D: NetDirectionType> NetworkMessages<D> {
    /// Creates a new `Messages` store. Doesn't allocate until [`push`](Self::push) is used.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
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
        // Add to the messages vec
        let idx = self.messages.len();
        self.messages.push(bytes);

        // Add index to the map
        self.queue_map
        .entry(channel)
        .or_insert(IdxVec::with_capacity(1))
        .push(idx);
    }

    /// Returns an iterator over channels, and their associated queues.
    pub fn iter(&self) -> ChannelIter {
        ChannelIter {
            messages: &self.messages,
            map_iter: self.queue_map.iter(),
        }
    }
}

impl<D: NetDirectionType> Default for NetworkMessages<D> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<D: NetDirectionType> std::fmt::Debug for NetworkMessages<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("NetworkMessages<{}>", std::any::type_name::<D>()))
    }
}

#[derive(Clone)]
pub struct ChannelIter<'a> {
    messages: &'a [Bytes],
    map_iter: std::collections::hash_map::Iter<'a, ChannelId, IdxVec>,
}

impl<'a> Iterator for ChannelIter<'a> {
    type Item = (ChannelId, MessageIter<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let (c, i) = self.map_iter.next()?;
        return Some((c.clone(), MessageIter {
            messages: &self.messages,
            indexes: i.as_slice(),
        }));
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.map_iter.size_hint()
    }
}

impl<'a> ExactSizeIterator for ChannelIter<'a> {}

#[derive(Clone)]
pub struct MessageIter<'a> {
    messages: &'a [Bytes],
    indexes: &'a [usize],
}

impl<'a> Iterator for MessageIter<'a> {
    type Item = Bytes;

    fn next(&mut self) -> Option<Self::Item> {
        // If there's no items left, return
        let ln = self.indexes.len();
        if ln == 0 { return None }

        // Get the next message we want
        let idx = self.indexes[0];
        let val = self.messages[idx].clone();

        // Change the slice to cut off the first item
        // This is cheaper than storing a cursor value
        self.indexes = &self.indexes[1..];

        // Return the message
        return Some(val);
    }
}

impl<'a> ExactSizeIterator for MessageIter<'a> {}