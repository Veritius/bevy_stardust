use std::{cmp::Ordering, collections::BTreeSet};
use bytes::Bytes;

use crate::sequences::sequence_greater_than;

/// Ensures items are popped in order, regardless of insertion order.
pub(crate) struct OrderedMessages {
    queue: Vec<OrderedMessage>,
}

impl OrderedMessages {
    pub fn new(reliable: bool) -> Self {
        Self {
            queue: Vec::with_capacity(16),
        }
    }

    pub fn pop(&mut self) -> Option<OrderedMessage> {
        todo!()
    }

    pub fn put(&mut self, message: OrderedMessage) {
        match self.queue.binary_search(&message) {
            Ok(_) => panic!(), // Shouldn't happen
            Err(idx) => {
                self.queue.insert(idx, message);
            },
        }
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