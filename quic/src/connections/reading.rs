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
    messages: &'a mut MessageQueue,
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

    pub fn opened(&mut self, id: StreamId) {
        self.readers.insert(id, Box::new(Recv::new()))
            .expect(&format!("A stream with id {id:?} already existed"));
    }

    pub fn reader<'a, S: ReadableStream>(
        &'a mut self,
        context: ParsingContext<'a>,
        buffers: IncomingBuffers<'a>,
        id: StreamId,
    ) -> IncomingStream {
        // Get or add Recv state from/to the map
        let recv = self.readers
            .entry(id)
            .or_insert_with(|| Box::new(Recv::new()))
            .as_mut();

        return IncomingStream { recv, context, buffers }
    }
}

pub(super) struct IncomingStream<'a> {
    recv: &'a mut Recv,
    context: ParsingContext<'a>,
    buffers: IncomingBuffers<'a>,
}

impl<'a> StreamReader for IncomingStream<'a> {
    fn read_from<S: ReadableStream>(&mut self, stream: &mut S) -> Result<usize, StreamReadError> {
        self.recv.read_from(stream)
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