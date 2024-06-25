use std::{collections::HashMap, marker::PhantomData};
use bevy::prelude::*;
use bytes::Bytes;
use smallvec::SmallVec;
use crate::prelude::*;
use super::direction::NetDirectionType;

type IdxVec = SmallVec<[usize; 2]>; 

/// A queue-like structure for storing messages, separated by channels.
/// 
/// The items in this queue **do not** persist across frames.
/// They are cleared in [`NetworkWrite::Clear`] in [`PostUpdate`].
#[derive(Component)]
pub struct NetworkMessages<D: NetDirectionType> {
    pub(crate) messages: Vec<Bytes>,
    pub(crate) index_map: HashMap<ChannelId, IdxVec>,
    phantom: PhantomData<D>
}

impl<D: NetDirectionType> NetworkMessages<D> {
    /// Creates a new `Messages` store. Doesn't allocate until [`push`](Self::push) is used.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            index_map: HashMap::new(),
            phantom: PhantomData,
        }
    }

    /// Clears all queues but doesn't reallocate.
    pub(crate) fn clear(&mut self) {
        // Clear the message map
        self.messages.clear();

        // Clear all indexes in the map
        self.index_map
        .iter_mut()
        .for_each(|(_, v)| v.clear())
    }

    /// Counts how many messages are queued in all channels.
    pub fn count(&self) -> usize {
        self.index_map
        .iter()
        .map(|(_,v)| v.len())
        .sum()
    }

    /// Pushes a new item to the queue.
    pub fn push(&mut self, channel: ChannelId, message: Bytes) {
        // Add to the messages vec
        let idx = self.messages.len();
        self.messages.push(message);

        // Add index to the map
        self.index_map
        .entry(channel)
        .or_insert(IdxVec::with_capacity(1))
        .push(idx);
    }

    /// Pushes messages from an iterator.
    /// This can be faster than calling [`push`](Self::push) repeatedly.
    /// The efficiency of this function depends on `I`'s implementation of [`size_hint`].
    /// 
    /// [`size_hint`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.size_hint
    pub fn push_many<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (ChannelId, Bytes)>,
    {
        // Convert the iterator and find the maximum expected size
        let iter = iter.into_iter();
        let size = match iter.size_hint() {
            (v, None) => v,
            (v, Some(a)) => v.max(a),
        };

        // Expand the message vector to fit, if necessary
        self.messages.reserve_exact(size.saturating_sub(self.messages.capacity()));

        // Push everything as per usual
        for (channel, payload) in iter {
            self.push(channel, payload);
        }
    }

    /// Pushes messages from an iterator to a single channel.
    /// This can be faster than calling [`push`](Self::push) or [`push_many`](Self::push_many) repeatedly.
    /// The efficiency of this function depends on `I`'s implementation of [`size_hint`].
    /// 
    /// [`size_hint`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.size_hint
    pub fn push_channel<I>(&mut self, channel: ChannelId, iter: I)
    where
        I: IntoIterator<Item = Bytes>,
    {
        // Convert the iterator and find the maximum expected size
        let iter = iter.into_iter();
        let size = match iter.size_hint() {
            (v, None) => v,
            (v, Some(a)) => v.max(a),
        };

        // Add index to the map
        let indexes = self.index_map
            .entry(channel)
            .or_insert(IdxVec::with_capacity(size));

        // Expand the vectors to fit, if necessary
        self.messages.reserve_exact(size.saturating_sub(self.messages.capacity()));
        indexes.reserve_exact(size.saturating_sub(indexes.capacity()));

        // Insert all payloads
        for payload in iter {
            let idx = self.messages.len();
            self.messages.push(payload);
            indexes.push(idx);
        }
    }

    /// Returns an iterator over channels, and their associated queues.
    pub fn iter(&self) -> ChannelIter {
        ChannelIter {
            messages: &self.messages,
            map_iter: self.index_map.iter(),
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

impl<D: NetDirectionType> Extend<(ChannelId, Bytes)> for NetworkMessages<D> {
    fn extend<T: IntoIterator<Item = (ChannelId, Bytes)>>(&mut self, iter: T) {
        iter.into_iter().for_each(|(c,p)| self.push(c, p));
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