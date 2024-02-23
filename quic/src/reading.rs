use std::collections::HashMap;
use bevy_ecs::prelude::*;
use bevy_stardust::{channels::{id::ChannelId, registry::ChannelRegistry}, connections::peer::NetworkPeer};
use quinn_proto::{Chunks, Dir, ReadError, ReadableError};
use crate::{streams::StreamErrorCode, QuicConnection};

pub(super) fn read_messages_from_streams_system(
    mut connections: Query<(Entity, &mut QuicConnection), With<NetworkPeer>>,
    registry: Res<ChannelRegistry>,
) {
    // Any processing that can run in parallel runs here
    connections.par_iter_mut().for_each(|(entity, mut connection)| {
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
            fn process_chunks(registry: &ChannelRegistry, chunks: &mut Chunks, stream_data: &mut IncomingStream) {
                // Iterate chunks until we run out or an error occurs
                loop { match chunks.next(usize::MAX) {
                    // We can read some data
                    Ok(Some(chunk)) => todo!(),

                    // Stream finished
                    Ok(None) => todo!(),

                    // Try to read again later
                    Err(ReadError::Blocked) => { break },

                    // Stream reset
                    Err(ReadError::Reset(error_code)) => todo!(),
                }}
            }

            // Start processing chunks :)
            process_chunks(&registry, &mut chunks, stream_data);
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
    pub fn put(&mut self, slice: &[u8]) {
        self.0.extend_from_slice(slice);
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