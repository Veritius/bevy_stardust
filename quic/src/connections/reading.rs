use bevy::{log::trace, utils::HashMap};
use bevy_stardust::messages::*;
use bytes::Bytes;
use quinn_proto::StreamId;
use crate::{datagrams::*, streams::*};

/// Shared context data used when parsing.
#[derive(Clone, Copy)]
pub(super) struct ParsingContext<'a> {
    pub channels: &'a ChannelRegistry,
}

pub(super) struct IncomingBuffers<'a> {
    messages: &'a mut MessageQueue,
}

pub(super) struct IncomingStreams {
    readers: HashMap<StreamId, Box<Recv>>,
}

impl IncomingStreams {
    pub fn recv<S: ReadableStream>(
        &mut self,
        context: ParsingContext,
        buffers: IncomingBuffers,
        id: StreamId,
        mut stream: S,
    ) {
        // Get or add Recv state from/to the map
        let recv = self.readers
            .entry(id)
            .or_insert_with(|| Box::new(Recv::new()))
            .as_mut();

        // Get the receiver to read the stream
        match recv.read_from(&mut stream) {
            Ok(_) => {},
            Err(error) => todo!(),
        }
    }
}

pub(super) struct IncomingDatagrams {
    desequencers: HashMap<ChannelId, DatagramDesequencer>,
}

impl IncomingDatagrams {
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
    inner: Vec<ChannelMessage>,
}