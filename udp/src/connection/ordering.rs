use std::cmp::Ordering;
use bytes::Bytes;
use crate::sequences::SequenceId;

/// Ensures items are popped in order, regardless of insertion order.
pub(crate) struct OrderedMessages {
    mode: OrderedMessagesMode,
    send_index: SequenceId,
    recv_queue: Vec<OrderedMessage>,
    recv_index: SequenceId,
}

impl OrderedMessages {
    pub fn new(mode: OrderedMessagesMode) -> Self {
        Self {
            mode,
            send_index: 0.into(),
            recv_queue: match mode {
                OrderedMessagesMode::Ordered => Vec::with_capacity(16),
                OrderedMessagesMode::Sequenced => Vec::new(), // never used
            },
            recv_index: 0.into(),
        }
    }

    pub fn recv(&mut self, message: OrderedMessage) -> Option<OrderedMessage> {
        match self.mode {
            // Sequenced messages are really simple.
            // If it's newer than the last one, return it.
            // Otherwise, don't do anything.
            OrderedMessagesMode::Sequenced => {
                if self.recv_index >= message.sequence {
                    self.recv_index = message.sequence + 1;
                    return Some(message);
                }
                return None;
            },

            OrderedMessagesMode::Ordered => {
                todo!()
            },
        }
    }

    pub fn advance(&mut self) -> SequenceId {
        let ind = self.send_index;
        self.send_index += 1;
        return ind;
    }
}

/// A message with `Eq` and `Ord` implementations based on `sequence`.
pub(crate) struct OrderedMessage {
    pub sequence: SequenceId,
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
        Some(self.cmp(other))
    }
}

impl Ord for OrderedMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sequence.cmp(&other.sequence)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum OrderedMessagesMode {
    Ordered,
    Sequenced,
}