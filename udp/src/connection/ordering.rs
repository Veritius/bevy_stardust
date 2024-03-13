use std::{cmp::Ordering, collections::BTreeSet};
use bytes::Bytes;

use crate::sequences::sequence_greater_than;

/// Ensures items are popped in order, regardless of insertion order.
pub(crate) struct OrderedMessages {
    highest: u16,
    queue: BTreeSet<OrderedMessage>,
    skipping: bool,
}

impl OrderedMessages {
    pub fn pop(&mut self) -> Option<OrderedMessage> {
        todo!()
    }

    pub fn put(&mut self, message: OrderedMessage) {
        self.queue.insert(message);
    }
}

/// A message with `Eq` and `Ord` implementations based on `sequence`.
pub(crate) struct OrderedMessage {
    pub sequence: u16,
    pub payload: Bytes,
}

impl PartialEq for OrderedMessage {
    fn eq(&self, other: &Self) -> bool {
        self.sequence == other.sequence
    }
}

impl Eq for OrderedMessage {}

impl PartialOrd for OrderedMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.sequence.partial_cmp(&other.sequence)
    }
}

impl Ord for OrderedMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}