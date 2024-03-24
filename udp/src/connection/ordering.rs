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
        recv_index: SequenceId,
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
            recv_index: SequenceId::default(),
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

            // Based on Laminar's ordering algorithm, adapted to use an ordered vec instead of a hashmap.
            // https://github.com/TimonPost/laminar/blob/e8ffb26a915bb6ac3c8d959031d63f8a776e763c/src/infrastructure/arranging/ordering.rs#L230-L242
            Self::Ordered {
                send_index: _,
                recv_queue,
                recv_index,
            } => {
                match (*recv_index).cmp(&message.sequence) {
                    Ordering::Less => {
                        return None;
                    },
                    Ordering::Equal => {
                        *recv_index += 1;
                        return Some(message);
                    },
                    Ordering::Greater => {
                        match recv_queue.binary_search(&message) {
                            Ok(_) => unreachable!(), // should be covered by the Equal case
                            Err(idx) => {
                                recv_queue.insert(idx, message);
                                return None;
                            },
                        }
                    },
                }
            }
        }
    }

    pub fn drain_available(&mut self) -> Option<impl Iterator<Item = OrderedMessage> + '_> {
        match self {
            OrderedMessages::Sequenced {
                send_index: _,
                recv_index: _,
            } => {
                return None;
            },

            // Based on Laminar's ordering algorithm, adapted to use an ordered vec instead of a hashmap.
            // https://github.com/TimonPost/laminar/blob/e8ffb26a915bb6ac3c8d959031d63f8a776e763c/src/infrastructure/arranging/ordering.rs#L266-L274
            OrderedMessages::Ordered {
                send_index: _,
                recv_queue,
                recv_index,
            } => {
                struct OrderedMessageDrain<'a> {
                    index: &'a mut SequenceId,
                    queue: &'a mut Vec<OrderedMessage>,
                }

                impl Iterator for OrderedMessageDrain<'_> {
                    type Item = OrderedMessage;
                    
                    fn next(&mut self) -> Option<Self::Item> {
                        match self.queue.binary_search(&OrderedMessage::search(*self.index)) {
                            Ok(idx) => {
                                *self.index += 1;
                                Some(self.queue.remove(idx))
                            },
                            Err(_) => None,
                        }
                    }
                }

                return Some(OrderedMessageDrain {
                    index: recv_index,
                    queue: recv_queue,
                });
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
                recv_index: _,
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

impl OrderedMessage {
    pub fn search(sequence: SequenceId) -> Self {
        Self {
            sequence,
            payload: Bytes::from_static(&[]),
        }
    }
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