use hashbrown::HashMap;
use smallvec::SmallVec;
use crate::prelude::*;

type IdxVec = SmallVec<[usize; 2]>; 

/// An efficient queue of messages, organised by channel.
pub struct MessageQueue {
    messages: Vec<Message>,
    indexes: HashMap<ChannelId, IdxVec>,
}

impl MessageQueue {
    /// Creates a new `Messages` store. Doesn't allocate until [`push_one`](Self::push_one) is used.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            indexes: HashMap::new(),
        }
    }

    /// Clears all queues but doesn't resize anything.
    /// 
    /// If you want to free allocations, use [`reset`](Self::reset).
    pub fn clear(&mut self) {
        // Clear the message map
        self.messages.clear();

        // Clear all indexes in the map
        self.indexes
        .iter_mut()
        .for_each(|(_, v)| v.clear())
    }

    /// Clears all data in the queue, and frees any allocations.
    pub fn reset(&mut self) {
        *self = MessageQueue::new();
    }

    /// Reserves capacity for at least `additional` new messages to be efficiently inserted.
    pub fn reserve(&mut self, additional: usize) {
        self.messages.reserve(additional);
    }

    /// Reserves capacity for at least `additional` new messages associated with `channel` to be efficiently inserted.
    pub fn reserve_channel(&mut self, channel: ChannelId, additional: usize) {
        // Reserve space in the messages vector
        self.reserve(additional);

        // Reserve space in the index vector
        self.indexes
        .entry(channel)
        .or_insert_with(|| SmallVec::new())
        .reserve(additional);
    }

    /// Returns the total number of messages stored in the queue.
    #[inline]
    pub fn count(&self) -> usize {
        self.messages.len()
    }

    /// Returns the sum of bytes from all messages in the queue.
    #[inline]
    pub fn bytes(&self) -> usize {
        self.messages.iter().map(|v| v.len()).sum()
    }

    /// Pushes a single message to the queue.
    pub fn push_one(&mut self, message: ChannelMessage) {
        // Add to the messages vec
        let idx = self.messages.len();
        self.messages.push(message.message);

        // Add index to the map
        self.indexes
        .entry(message.channel)
        .or_insert(IdxVec::with_capacity(1))
        .push(idx);
    }

    /// Pushes messages from `iter` to the queue.
    /// This can be faster than calling [`push_one`](Self::push_one) repeatedly.
    pub fn push_many<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = ChannelMessage>,
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
        for message in iter {
            self.push_one(message);
        }
    }

    /// Pushes many messages from `iter` to a single channel.
    /// This can be faster than calling [`push_one`](Self::push_one) or [`push_many`](Self::push_many) repeatedly.
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
        let indexes = self.indexes
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
            map_iter: self.indexes.iter(),
        }
    }

    /// Returns an iterator over all messages in a specific channel.
    pub fn iter_channel(&self, channel: ChannelId) -> MessageIter {
        match self.indexes.get(&channel) {
            // The index map exists, return a real MessageIter
            Some(indexes) => MessageIter {
                messages: &self.messages,
                indexes: indexes.as_slice(),
            },

            // MessageIter tracks progress by shrinking a slice every iteration.
            // If the slice we give it is empty, it instantly returns None.
            // By returning an iterator that ends instantly, we don't need to
            // make this function have a return type of Option<MessageIter>.
            None => MessageIter {
                messages: &self.messages,
                indexes: &[],
            },
        }
    }
}

impl Default for MessageQueue {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Extend<ChannelMessage> for MessageQueue {
    #[inline]
    fn extend<T: IntoIterator<Item = ChannelMessage>>(&mut self, iter: T) {
        self.push_many(iter);
    }
}

impl<'a> IntoIterator for &'a MessageQueue {
    type IntoIter = ChannelIter<'a>;
    type Item = <ChannelIter<'a> as Iterator>::Item;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over individual channels in a [`MessageQueue`].
/// 
/// Produces [`ChannelId`] values, and [`MessageIter`] iterators.
/// The order of iteration over channels is unspecified, and may change unpredictably.
#[derive(Clone)]
pub struct ChannelIter<'a> {
    messages: &'a [Message],
    map_iter: hashbrown::hash_map::Iter<'a, ChannelId, IdxVec>,
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

/// An iterator over individual messages in channel, produced by a [`ChannelIter`].
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

#[test]
fn message_queue_ordering_test() {
    let mut queue = MessageQueue::new();

    fn map_messages(set: &'static [&'static [u8]]) -> impl Iterator<Item = Message> {
        set.iter().map(|v| Message::from_bytes(Bytes::from_static(v)))
    }

    const MESSAGE_SET_A: &[&[u8]] = &[
        b"Hello, world!",
        b"It's a very nice day, isn't it?",
        b"Yeah, I agree!",
    ];

    const MESSAGE_SET_B: &[&[u8]] = &[
        b"Hello, world!",
        b"It's a miserable day, isn't it?",
        b"No, I think it's fine!",
    ];

    const MESSAGE_SET_C: &[&[u8]] = &[
        b"Hello, world!",
        b"It's an alright day, isn't it?",
        b"That's a good way of putting it.",
    ];

    // Add all the messages to the channel
    queue.push_channel(ChannelId::from(0), map_messages(MESSAGE_SET_A));
    queue.push_channel(ChannelId::from(0), map_messages(MESSAGE_SET_B));
    queue.push_channel(ChannelId::from(1), map_messages(MESSAGE_SET_C));

    queue.iter_channel(ChannelId::from(0))
    .zip(MESSAGE_SET_A.iter().chain(MESSAGE_SET_B))
    .for_each(|(a, b)| assert_eq!(a.as_slice(), *b));

    queue.iter_channel(ChannelId::from(1))
    .zip(MESSAGE_SET_C)
    .for_each(|(a, b)| assert_eq!(a.as_slice(), *b));
}