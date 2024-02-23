use std::collections::HashMap;
use bevy_ecs::prelude::*;
use bevy_stardust::{channels::{id::ChannelId, registry::ChannelRegistry}, connections::peer::NetworkPeer};
use quinn_proto::{Chunks, Dir, ReadError, ReadableError, StreamId};
use crate::{streams::StreamErrorCode, QuicConnection};

pub(super) fn read_messages_from_streams_system(
    mut connections: Query<(Entity, &mut QuicConnection), With<NetworkPeer>>,
    registry: Res<ChannelRegistry>,
) {
    // Any processing that can run in parallel runs here
    connections.par_iter_mut().for_each(|(entity, mut connection)| {
        // Scratch space for some operations
        let mut scratch: Vec<u8> = Vec::with_capacity(512); // 512 seems like a reasonable default

        // Accept all new streams
        while let Some(stream_id) = connection.inner.get_mut().streams().accept(Dir::Uni) {
            connection.recv_streams.insert(stream_id, IncomingStream::default());
        }

        // Split borrow function to help out borrowck
        // Mutably borrowing multiple struct fields is not valid apparently
        #[inline]
        fn split_borrow(connection: &mut QuicConnection) -> (
            &mut quinn_proto::Connection,
            &mut HashMap<quinn_proto::StreamId, IncomingStream>,
        ) {(
            connection.inner.get_mut(),
            &mut connection.recv_streams
        )}

        // Borrow many fields using the split borrow function
        let (
            connection_inner,
            active_recv_streams,
        ) = split_borrow(&mut connection);

        // Iterate over all known streams and handle incoming data
        let mut remove_streams = Vec::new();
        for (stream_id, stream_data) in active_recv_streams.iter_mut() {
            let mut recv_stream = connection_inner.recv_stream(*stream_id);

            // Try to get our chunk reader
            let mut chunks = match recv_stream.read(true) {
                // No problems getting the chunks
                Ok(read) => read,

                // We can't read anymore so just remove it from the map
                Err(ReadableError::UnknownStream) => {
                    todo!()
                },

                // This shouldn't happen, so it just panics.
                Err(ReadableError::IllegalOrderedRead) => panic!(),
            };

            // This is a function so we can change the stream state
            // and then go back to processing it, since we're in a for loop
            fn process_chunks(
                registry: &ChannelRegistry,
                removes: &mut Vec<StreamId>,
                scratch: &mut Vec<u8>,
                chunks: &mut Chunks,
                stream_data: &mut IncomingStream,
            ) {
                // Iterate chunks until we run out or an error occurs
                loop { match chunks.next(usize::MAX) {
                    // We can read some data
                    Ok(Some(chunk)) => {
                        stream_data.buffer.insert_back(&chunk.bytes);
                        let mut reader = untrusted::Reader::new(untrusted::Input::from(stream_data.buffer.read_slice()));

                        match &mut stream_data.data {
                            IncomingStreamData::PendingPurpose => todo!(),
                            IncomingStreamData::ConnectionManagement => todo!(),
                            IncomingStreamData::StardustPayloads { id } => todo!(),
                            IncomingStreamData::NeedsClosing { reason } => todo!(),
                        }
                    },

                    // Stream finished
                    Ok(None) => todo!(),

                    // Try to read again later
                    Err(ReadError::Blocked) => { break },

                    // Stream reset
                    Err(ReadError::Reset(error_code)) => todo!(),
                }}
            }

            // Start processing chunks :)
            process_chunks(
                &registry,
                &mut remove_streams,
                &mut scratch,
                &mut chunks,
                stream_data,
            );
        }
    });
}

pub(crate) struct IncomingStream {
    buffer: IncomingStreamBuffer,
    data: IncomingStreamData,
}

impl Default for IncomingStream {
    fn default() -> Self {
        Self {
            buffer: IncomingStreamBuffer(Vec::with_capacity(32)),
            data: IncomingStreamData::PendingPurpose,
        }
    }
}

struct IncomingStreamBuffer(Vec<u8>);

impl IncomingStreamBuffer {
    fn read_slice(&self) -> &[u8] {
        &self.0
    }

    fn insert_back(&mut self, slice: &[u8]) {
        self.0.extend_from_slice(slice);
    }

    fn remove_front(&mut self, scratch: &mut Vec<u8>, amount: usize) {
        // Check we're not wasting any time
        if amount >= self.0.len() {
            self.0.clear();
        }

        // Ensure scratch is the right size for the operation
        let min_size = self.0.len() - amount;
        if scratch.len() < min_size {
            scratch.reserve_exact(min_size - scratch.len());
        }

        // Use scratch to remove a section of ourselves
        scratch.clear();
        scratch.clone_from_slice(&self.0[amount..]);
        self.0.clear();
        self.0.clone_from_slice(&scratch);
    }
}

enum IncomingStreamData {
    PendingPurpose,
    ConnectionManagement,
    StardustPayloads {
        id: ChannelId,
    },
    NeedsClosing {
        reason: StreamErrorCode
    },
}