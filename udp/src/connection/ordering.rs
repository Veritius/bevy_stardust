use std::{cmp::Ordering, collections::BTreeMap, fmt::Debug};
use bevy_stardust::{messages::channels::ChannelId, prelude::{ChannelConfiguration, ChannelConsistency}};
use bytes::Bytes;
use crate::sequences::SequenceId;

// Storage for ordered messages.
pub(crate) struct OrderingManager {
    stardust_messages: BTreeMap<ChannelId, OrderedMessages>,
}

impl OrderingManager {
    pub fn new() -> Self {
        Self {
            stardust_messages: BTreeMap::default(),
        }
    }

    pub fn get(&mut self, channel: ChannelId, config: &ChannelConfiguration) -> &mut OrderedMessages {
        self.stardust_messages
        .entry(channel)
        .or_insert_with(|| {
            match config.consistency {
                ChannelConsistency::ReliableOrdered => OrderedMessages::ordered(),
                ChannelConsistency::UnreliableSequenced => OrderedMessages::sequenced(),
                _ => panic!("Can't make an OrderedMessages for an unordered channel"),
            }
        })
    }
}

/// Ensures items are popped in order, regardless of insertion order.
pub(crate) struct OrderedMessages {
    mode: OrderingMode,
    send_index: SequenceId,
    recv_queue: Vec<OrderedMessage>,
    recv_index: SequenceId,
}

impl OrderedMessages {
    pub fn sequenced() -> Self {
        Self {
            mode: OrderingMode::Sequenced,
            send_index: SequenceId::default(),
            recv_queue: Vec::with_capacity(0), // never used
            recv_index: SequenceId::default(),
        }
    }

    pub fn ordered() -> Self {
        Self {
            mode: OrderingMode::Ordered,
            send_index: SequenceId::default(),
            recv_queue: Vec::with_capacity(0),
            recv_index: SequenceId::default(),
        }
    }

    pub fn recv(&mut self, message: OrderedMessage) -> Option<OrderedMessage> {
        match self.mode {
            // Sequenced messages are really simple.
            // If it's newer than the last one, return it.
            // Otherwise, don't do anything.
            OrderingMode::Sequenced => {
                if self.recv_index >= message.sequence {
                    self.recv_index = message.sequence + 1;
                    return Some(message);
                }
                return None;
            },

            // Based on Laminar's ordering algorithm, adapted to use an ordered vec instead of a hashmap.
            // https://github.com/TimonPost/laminar/blob/e8ffb26a915bb6ac3c8d959031d63f8a776e763c/src/infrastructure/arranging/ordering.rs#L230-L242
            OrderingMode::Ordered => {
                match message.sequence.cmp(&self.recv_index) {
                    Ordering::Less => {
                        return None;
                    },
                    Ordering::Equal => {
                        self.recv_index += 1;
                        return Some(message);
                    },
                    Ordering::Greater => {
                        match self.recv_queue.binary_search(&message) {
                            Ok(idx) => {
                                // Shouldn't happen.
                                self.recv_queue.remove(idx);
                                return None;
                            }
                            Err(idx) => {
                                self.recv_queue.insert(idx, message);
                                return None;
                            },
                        }
                    },
                }
            }
        }
    }

    pub fn drain_available(&mut self) -> Option<impl Iterator<Item = OrderedMessage> + '_> {
        match self.mode {
            // Sequenced ordering never stores anything in the queue.
            OrderingMode::Sequenced => { return None; }

            // Based on Laminar's ordering algorithm, adapted to use an ordered vec instead of a hashmap.
            // https://github.com/TimonPost/laminar/blob/e8ffb26a915bb6ac3c8d959031d63f8a776e763c/src/infrastructure/arranging/ordering.rs#L266-L274
            OrderingMode::Ordered => {
                struct OrderedMessageDrain<'a> {
                    index: &'a mut SequenceId,
                    queue: &'a mut Vec<OrderedMessage>,
                }

                impl Iterator for OrderedMessageDrain<'_> {
                    type Item = OrderedMessage;
                    
                    fn next(&mut self) -> Option<Self::Item> {
                        match self.queue.binary_search(&OrderedMessage::blank(*self.index)) {
                            Ok(idx) => {
                                *self.index += 1;
                                Some(self.queue.remove(idx))
                            },
                            Err(_) => None,
                        }
                    }
                }

                return Some(OrderedMessageDrain {
                    index: &mut self.recv_index,
                    queue: &mut self.recv_queue,
                });
            }
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

impl OrderedMessage {
    pub fn blank(sequence: SequenceId) -> Self {
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

impl Debug for OrderedMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderedMessage")
        .field("sequence", &self.sequence)
        .finish()
    }
}

enum OrderingMode {
    Sequenced,
    Ordered,
}

mod tests {
    #![allow(unused_imports)]

    use super::*;

    #[test]
    fn sequenced_messages_test() {
        let mut state = OrderedMessages::sequenced();
        assert_eq!(state.recv(OrderedMessage::blank(0.into())), Some(OrderedMessage::blank(0.into())));
        assert_eq!(state.recv(OrderedMessage::blank(1.into())), Some(OrderedMessage::blank(1.into())));
        assert_eq!(state.recv(OrderedMessage::blank(3.into())), Some(OrderedMessage::blank(3.into())));
        assert_eq!(state.recv(OrderedMessage::blank(2.into())), None);
    }

    #[test]
    fn ordered_messages_test() {
        let mut state = OrderedMessages::ordered();
        assert_eq!(state.recv(OrderedMessage::blank(0.into())), Some(OrderedMessage::blank(0.into())));
        assert_eq!(state.recv(OrderedMessage::blank(1.into())), Some(OrderedMessage::blank(1.into())));
        assert_eq!(state.recv(OrderedMessage::blank(3.into())), None);
        assert_eq!(state.recv(OrderedMessage::blank(4.into())), None);
        assert_eq!(state.recv(OrderedMessage::blank(2.into())), Some(OrderedMessage::blank(2.into())));

        let re = state.drain_available().unwrap().collect::<Vec<_>>();
        assert_eq!(re, vec![
            OrderedMessage::blank(3.into()),
            OrderedMessage::blank(4.into()),
        ]);
    }
}