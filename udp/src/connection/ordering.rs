use std::cmp::Ordering;
use bytes::Bytes;

/// Ensures items are popped in order, regardless of insertion order.
pub(crate) struct OrderedMessages {
    mode: OrderedMessagesMode,
    send_index: u16,
    recv_queue: Vec<OrderedMessage>,
    recv_index: u16,
}

impl OrderedMessages {
    pub fn new(mode: OrderedMessagesMode) -> Self {
        Self {
            mode,
            send_index: 0,
            recv_queue: Vec::with_capacity(16),
            recv_index: 0,
        }
    }

    pub fn recv(&mut self, message: OrderedMessage) -> Option<OrderedMessage> {
        todo!()
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

pub(crate) enum OrderedMessagesMode {
    Ordered,
    Sequenced,
}