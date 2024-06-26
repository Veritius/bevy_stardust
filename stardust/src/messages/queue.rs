use std::{collections::HashMap, marker::PhantomData};
use bevy::prelude::*;
use smallvec::SmallVec;
use crate::prelude::*;
use super::direction::NetDirectionType;

type IdxVec = SmallVec<[usize; 2]>; 

/// A queue of [messages](Message), organised by channel.
/// When added to [`NetworkPeer`] entities, it is the set of messages related to them.
/// Items in this queue are not shared. If you want to send a message to multiple peers,
/// you must push it manually to each queue.
/// 
/// This queue is cleared every tick in [`PostUpdate`], in the [`NetworkWrite::Clear`] system set.
/// Since a [`Message`] is a reference-counting type, that allocation may remain if used elsewhere.
/// Note that the clearing is done by a system, so if it's not in the `World`, it will not be cleared.
/// 
/// # Direction
/// The `D` generic in `NetworkMessages<D>` is the 'direction'.
/// This makes it so that there are two instances of this component per peer,
/// where each serves a different purpose in networking code, as well as aiding concurrency.
/// 
/// | Direction | Purpose                                     | Game systems | Transport layers |
/// |-----------|---------------------------------------------|--------------|------------------|
/// | Incoming  | Iterator over newly received messages       | Read only    | Write only       |
/// | Outgoing  | Queue for messages that must be transmitted | Write only   | Read only        |
#[derive(Component)]
pub struct NetworkMessages<D: NetDirectionType> {
    messages: Vec<Message>,
    index_map: HashMap<ChannelId, IdxVec>,
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
    #[inline]
    pub fn count(&self) -> usize {
        self.messages.len()
    }

    /// Returns the total amount of bytes queued to be sent.
    #[inline]
    pub fn bytes(&self) -> usize {
        self.messages.iter().map(|v| v.len()).sum()
    }

    /// Pushes a new item to the queue.
    pub fn push(&mut self, channel: ChannelId, message: Message) {
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
    pub fn push_many<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (ChannelId, Message)>,
    {
        // Convert the iterator and find the maximum expected size
        let iter = iter.into_iter();
        let size = match iter.size_hint() {
            (v, None) => v,
            (v, Some(a)) => v.max(a),
        };

        // Expand the message vector to fit, if necessary
        self.messages.reserve(size);

        // Push everything as per usual
        for (channel, payload) in iter {
            self.push(channel, payload);
        }
    }

    /// Pushes messages from an iterator to a single channel.
    /// This can be faster than calling [`push`](Self::push) or [`push_many`](Self::push_many) repeatedly.
    pub fn push_channel<I>(&mut self, channel: ChannelId, iter: I)
    where
        I: IntoIterator<Item = Message>,
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
        self.messages.reserve(size);
        indexes.reserve(size);

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

impl<D: NetDirectionType> Extend<(ChannelId, Message)> for NetworkMessages<D> {
    #[inline]
    fn extend<T: IntoIterator<Item = (ChannelId, Message)>>(&mut self, iter: T) {
        self.push_many(iter);
    }
}

impl<'a, D: NetDirectionType> IntoIterator for &'a NetworkMessages<D> {
    type IntoIter = ChannelIter<'a>;
    type Item = <ChannelIter<'a> as Iterator>::Item;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over individual channels in a [`NetworkMessages<D>`] component.
/// 
/// Produces `ChannelId` values, and [`MessageIter`] iterators.
/// The order of iteration over channels is unspecified, and may change unpredictably.
#[derive(Clone)]
pub struct ChannelIter<'a> {
    messages: &'a [Message],
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

/// An iterator over individual messages in channel, produced by a [`ChannelIter`] from an [`NetworkMessages<D>`] component.
/// 
/// Produces the contents of the messages in the order they were added to the queue.
#[derive(Clone)]
pub struct MessageIter<'a> {
    messages: &'a [Message],
    indexes: &'a [usize],
}

impl<'a> Iterator for MessageIter<'a> {
    type Item = Message;

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

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.indexes.len();
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for MessageIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.indexes.len()
    }
}

pub(crate) fn clear_message_queue_system<D: NetDirectionType>(
    mut queues: Query<&mut NetworkMessages<D>, Changed<NetworkMessages<D>>>,
) {
    queues.par_iter_mut().for_each(|mut queue| {
        queue.clear();
    });
}