use std::cmp::Ordering;
use bytes::Bytes;

use crate::sequences::sequence_greater_than;

/// Ensures items are popped in order, regardless of insertion order.
pub(crate) struct OrderedMessages {
    send_index: u16,
    recv_queue: Vec<OrderedMessage>,
}

impl OrderedMessages {
    pub fn new() -> Self {
        Self {
            recv_queue: Vec::with_capacity(16),
            send_index: 0,
        }
    }

    pub fn pop(&mut self) -> Option<OrderedMessage> {
        todo!()
    }

    pub fn put(&mut self, message: OrderedMessage) {
        match self.recv_queue.binary_search(&message) {
            Ok(_) => panic!(), // Shouldn't happen
            Err(idx) => {
                self.recv_queue.insert(idx, message);
            },
        }
    }

    pub fn advance(&mut self) -> u16 {
        let ind = self.send_index;
        self.send_index = self.send_index.wrapping_add(1);
        return ind;
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