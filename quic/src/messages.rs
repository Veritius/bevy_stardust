use std::collections::BTreeMap;

use bevy_stardust::{channels::ChannelRegistry, messages::MessageQueue, prelude::*};
use bevy_stardust_extras::numbers::Sequence;
use crate::{segments::{Header, Segment}, Connection};

/// Context object required to handle outgoing messages.
#[derive(Clone, Copy)]
pub struct SendContext<'a> {
    /// A reference to the application's channel registry.
    pub registry: &'a ChannelRegistry,

    /// The maximum size of sent datagrams.
    pub dgram_max_size: usize,
}

impl Connection {
    /// Handle outgoing messages from a [`PeerMessages<Outgoing>`] component.
    #[inline]
    pub fn handle_outgoing<'a>(
        &'a mut self,
        context: SendContext<'a>,
        queue: &'a PeerMessages<Outgoing>,
    ) {
        self.handle_outgoing_queue(
            context,
            queue.as_ref()
        )
    }

    /// Handles outgoing messages from a [`MessageQueue`].
    /// 
    /// If possible, you should use [`handle_outgoing`](Self::handle_outgoing) instead.
    pub fn handle_outgoing_queue<'a>(
        &'a mut self,
        context: SendContext<'a>,
        queue: &'a MessageQueue,
    ) {
        for (channel, messages) in queue.iter() {
            self.handle_outgoing_iter(context, channel, messages);
        }
    }

    /// Handles outgoing messages on a specific channel from an iterator.
    pub fn handle_outgoing_iter<'a, I>(
        &'a mut self,
        context: SendContext<'a>,
        channel: ChannelId,
        iter: I,
    ) where
        I: IntoIterator<Item = Message>,
    {
        let config = match context.registry.config(channel) {
            Some(config) => config,
            None => panic!("Channel {channel:?} did not exist in the registry"),
        };

        match config.consistency {
            MessageConsistency::UnreliableUnordered => {
                for message in iter {
                    self.handle_outgoing_unrel_unord(context, channel, message);
                }
            },

            MessageConsistency::UnreliableSequenced => {
                for message in iter {
                    self.handle_outgoing_unrel_seq(context, channel, message);
                }
            },

            MessageConsistency::ReliableUnordered => {
                for message in iter {
                    self.stream_segment_transient(Segment {
                        header: Header::UnorderedMessage { channel },
                        payload: message.into(),
                    });
                }
            },

            MessageConsistency::ReliableOrdered => {
                let id = self.get_channel_stream(channel);
                self.stream_segment_existing_iter(id, iter.into_iter().map(|message| {
                    Segment {
                        header: Header::UnorderedMessage { channel },
                        payload: message.into(),
                    }
                }));
            },

            _ => unimplemented!(),
        }
    }

    fn handle_outgoing_unrel_unord<'a>(
        &'a mut self,
        context: SendContext<'a>,
        channel: ChannelId,
        message: Message,
    ) {
        // Create the datagram header
        let header = Header::UnorderedMessage { channel };
        self.send_segment(Segment { header, payload: message.into() }, context.dgram_max_size);
    }

    fn handle_outgoing_unrel_seq<'a>(
        &'a mut self,
        context: SendContext<'a>,
        channel: ChannelId,
        message: Message,
    ) {
        // Get the sequence values
        // TODO: Don't lookup every time we want to send a message
        let sq_mgr = self.local_message_sequence(channel);

        // Create the datagram header
        let header = Header::SequencedMessage {
            channel,
            sequence: sq_mgr.next(),
        };

        // Send the datagram
        self.send_segment(Segment { header, payload: message.into() }, context.dgram_max_size);
    }

    pub(crate) fn local_message_sequence(&mut self, channel: ChannelId) -> &mut MessageSequence {
        self.message_sequences.local.entry(channel)
            .or_insert_with(|| MessageSequence::new())
    }
}

pub(crate) struct MessageSequenceMap {
    pub local: BTreeMap<ChannelId, MessageSequence>,
    pub remote: BTreeMap<ChannelId, MessageSequence>,
}

impl MessageSequenceMap {
    pub fn new() -> Self {
        Self {
            local: BTreeMap::new(),
            remote: BTreeMap::new(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct MessageSequence(Sequence<u16>);

impl MessageSequence {
    pub fn new() -> Self {
        Self(Sequence::from(0))
    }

    pub fn next(&mut self) -> Sequence<u16> {
        let v = self.0;
        self.0.increment();
        return v;
    }

    pub fn latest(&mut self, index: Sequence<u16>) -> bool {
        if index > self.0 {
            self.0 = index;
            return true;
        } else {
            return false;
        }
    }
}