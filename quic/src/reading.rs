use std::collections::HashMap;
use bevy_ecs::prelude::*;
use bevy_stardust::{channels::registry::ChannelRegistry, connections::{groups::NetworkGroup, peer::NetworkPeer}};
use quinn_proto::{Chunks, Dir, ReadError, ReadableError};
use crate::{streams::IncomingStream, QuicConnection};

const UNVERIFIED_BUFFER_SIZE: usize = 32;

pub(super) fn read_messages_from_streams_system(
    network_groups: Query<&NetworkGroup>,
    mut connections: Query<(Entity, &mut QuicConnection), With<NetworkPeer>>,
    registry: Res<ChannelRegistry>,
) {
    // Any processing that can run in parallel runs here
    connections.par_iter_mut().for_each(|(entity, mut connection)| {
        // Accept all new streams
        while let Some(stream_id) = connection.inner.get_mut().streams().accept(Dir::Uni) {
            connection.recv_streams.insert(stream_id, IncomingStream::Unverified { buffer: Vec::with_capacity(UNVERIFIED_BUFFER_SIZE) });
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
                    *stream_data = IncomingStream::NeedsRemoval { reason: None };
                    continue // move on to the next stream
                },

                // This shouldn't happen, so it just panics.
                Err(ReadableError::IllegalOrderedRead) => panic!(),
            };

            // This is a function so we can change the stream state
            // and then go back to processing it, since we're in a for loop
            fn process_chunks(chunks: &mut Chunks, stream_data: &mut IncomingStream) {
                // Iterate chunks until we run out or an error occurs
                loop { match chunks.next(usize::MAX) {
                    // We can read some data
                    Ok(Some(chunk)) => match stream_data {
                        IncomingStream::Unverified { buffer } => todo!(),
                        IncomingStream::ConnectionManagement { } => todo!(),
                        IncomingStream::StardustChannel { id, buffer } => todo!(),
                        IncomingStream::NeedsRemoval { reason } => todo!(),
                    },

                    // Stream finished
                    Ok(None) => match stream_data {
                        IncomingStream::Unverified { buffer } => todo!(),
                        IncomingStream::ConnectionManagement { } => todo!(),
                        IncomingStream::StardustChannel { id, buffer } => todo!(),
                        IncomingStream::NeedsRemoval { reason } => todo!(),
                    },

                    // Try to read again later
                    Err(ReadError::Blocked) => { break },

                    // Stream reset
                    Err(ReadError::Reset(error_code)) => todo!(),
                }}
            }

            // Start processing chunks :)
            process_chunks(&mut chunks, stream_data);
        }
    });
}