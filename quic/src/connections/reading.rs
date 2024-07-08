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

pub(super) struct IncomingStreams {
    readers: HashMap<StreamId, Box<Recv>>,
}

impl IncomingStreams {
    pub fn recv<S: ReadableStream>(
        &mut self,
        id: StreamId,
        mut stream: S,
    ) {
        // Repeatedly call read on the stream
        loop { match stream.read() {
            // A chunk was returned, read it
            StreamReadOutcome::Chunk(chunk) => {
                todo!()
            },

            // No more data to read, break the loop
            StreamReadOutcome::Blocked => break,

            // The stream is finished
            StreamReadOutcome::Finished => {
                todo!()
            },

            // The stream ended early due to an error
            StreamReadOutcome::Error(_) => todo!(),
        }
    } }
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