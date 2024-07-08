use bevy::{log::trace, utils::HashMap};
use bevy_stardust::messages::*;
use bytes::Bytes;
use quinn_proto::StreamId;
use crate::{datagrams::*, streams::*, QuicConfig};

/// Shared context data used when parsing.
#[derive(Clone, Copy)]
pub(super) struct ParsingContext<'a> {
    pub config: &'a QuicConfig,
    pub channels: &'a ChannelRegistry,
}

pub(super) struct IncomingBuffers<'a> {
    pub messages: &'a mut MessageQueue,
}

pub(super) struct IncomingStreams {
    readers: HashMap<StreamId, Box<Recv>>,
}

impl IncomingStreams {
    pub fn new() -> Self {
        Self {
            readers: HashMap::new(),
        }
    }

    pub fn read<'a, S: ReadableStream>(
        &'a mut self,
        context: ParsingContext<'a>,
        buffers: IncomingBuffers<'a>,
        id: StreamId,
        mut stream: S,
    ) {
        // Get or add Recv state from/to the map
        let recv = self.readers
            .entry(id)
            .or_insert_with(|| Box::new(Recv::new()))
            .as_mut();

        // Read data into the Recv state
        match recv.read_from(&mut stream) {
            Ok(_) => {},
            Err(_) => todo!(),
        };

        // Check if the Recv is ready to be read
        if !recv.ready() { return }
        let header = recv.header().unwrap();
        let iter = recv.iter(context.config.maximum_framed_message_length).unwrap();

        // Repeatedly pull from the Recv
        for item in iter {
            let payload = match item {
                Ok(d) => d,
                Err(_) => break,
            };

            match header {
                StreamHeader::Stardust { channel } => {
                    buffers.messages.push_one(ChannelMessage {
                        channel: ChannelId::from(channel),
                        payload: Message::from_bytes(payload),
                    });
                },
            }
        }
    }

    pub fn remove(&mut self, id: StreamId) {
        self.readers.remove(&id);
    }
}

pub(super) struct IncomingDatagrams {
    desequencers: HashMap<ChannelId, DatagramDesequencer>,
}

impl IncomingDatagrams {
    pub fn new() -> Self {
        Self {
            desequencers: HashMap::new(),
        }
    }

    pub fn recv(
        &mut self,
        context: ParsingContext,
        datagram: Bytes,
    ) {
        // Decode the datagram header and related stuff
        let datagram = match Datagram::decode(&mut datagram.clone()) {
            Ok(datagram) => datagram,
            Err(err) => {
                trace!("Error while decoding datagram: {err:?}");
                return;
            },
        };

        // Validate the datagram based on its header
        match datagram.header.purpose {
            DatagramPurpose::StardustUnordered { channel } => todo!(),
            DatagramPurpose::StardustSequenced { channel, sequence } => todo!(),
        }
    }
}

pub(super) struct HeldMessages {
    queue: MessageQueue,
}

impl HeldMessages {
    pub fn new() -> Self {
        Self {
            queue: MessageQueue::new(),
        }
    }

    #[inline]
    pub fn inner(&mut self) -> &mut MessageQueue {
        &mut self.queue
    }
}