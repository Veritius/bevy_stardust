use std::cmp::Ordering;
use bytes::Bytes;
use crate::sequences::SequenceId;

/// Ensures items are popped in order, regardless of insertion order.
pub(crate) enum OrderedMessages {
    Sequenced {
        send_index: SequenceId,
        recv_index: SequenceId,
    },
    Ordered {
        send_index: SequenceId,
        recv_queue: Vec<OrderedMessage>,
        oldest: SequenceId,
        newest: SequenceId,
    },
}

impl OrderedMessages {
    pub fn sequenced() -> Self {
        Self::Sequenced {
            send_index: SequenceId::default(),
            recv_index: SequenceId::default(),
        }
    }

    pub fn ordered() -> Self {
        Self::Ordered {
            send_index: SequenceId::default(),
            recv_queue: Vec::with_capacity(16),
            oldest: SequenceId::default(),
            newest: SequenceId::default(),
        }
    }

    pub fn recv(&mut self, message: OrderedMessage) -> Option<OrderedMessage> {
        match self {
            // Sequenced messages are really simple.
            // If it's newer than the last one, return it.
            // Otherwise, don't do anything.
            Self::Sequenced {
                send_index: _,
                recv_index
            } => {
                if *recv_index >= message.sequence {
                    *recv_index = message.sequence + 1;
                    return Some(message);
                }
                return None;
            },

            Self::Ordered {
                send_index,
                recv_queue,
                oldest,
                newest,
            } => {
                todo!()
            }
        }
    }

    pub fn drain_available(&mut self) -> Option<() /* impl Iterator<Item = OrderedMessage> */> {
        match self {
            OrderedMessages::Sequenced {
                send_index: _,
                recv_index: _,
            } => {
                return None;
            },

            OrderedMessages::Ordered {
                send_index: _,
                recv_queue: _,
                oldest: _,
                newest: _,
            } => {
                todo!()
            }
        }
    }

    pub fn advance(&mut self) -> SequenceId {
        match self {
            OrderedMessages::Sequenced {
                send_index,
                recv_index: _,
            } => {
                let ind = *send_index;
                *send_index += 1;
                return ind;
            },

            OrderedMessages::Ordered {
                send_index,
                recv_queue: _,
                oldest: _,
                newest: _,
            } => {
                let ind = *send_index;
                *send_index += 1;
                return ind;
            },
        }
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