use std::collections::HashMap;
use bevy_ecs::prelude::*;
use bevy_stardust::{channels::{id::ChannelId, registry::ChannelRegistry}, connections::peer::NetworkPeer};
use quinn_proto::{Chunks, Dir, ReadError, ReadableError};
use untrusted::{Input, Reader};
use crate::{streams::{StreamErrorCode, StreamPurposeHeader}, QuicConnection};

pub(super) fn read_messages_from_streams_system(
    mut connections: Query<(Entity, &mut QuicConnection), With<NetworkPeer>>,
    registry: Res<ChannelRegistry>,
) {
    // Any processing that can run in parallel runs here
    connections.par_iter_mut().for_each(|(entity, mut connection)| {
        // Scratch space for some operations
        let mut scratch: Vec<u8> = Vec::with_capacity(512); // 512 seems like a reasonable default
        let scratch = &mut scratch;

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

            'rloop: loop {
                match chunks.next(0) {
                    // A chunk of bytes was received for processing.
                    Ok(Some(chunk)) => {
                        // Buffer reader
                        stream_data.buffer.insert_back(&chunk.bytes);
                        let mut reader = Reader::new(Input::from(stream_data.buffer.read_slice()));
                        let mut pending_removals = 0;

                        // Process individual pieces of information within the chunk
                        'iloop: loop {
                            match &mut stream_data.data {
                                // The initial data is the packet
                                IncomingStreamData::PendingPurpose(_) => {
                                    // Get the purpose header that should be at the start of new streams
                                    let purpose_header = match reader.read_byte().ok() {
                                        Some(val) => match StreamPurposeHeader::try_from(val).ok() {
                                            Some(val) => val,
                                            None => { todo!() },
                                        },
                                        None => { todo!() },
                                    };
    
                                    // Turn the purpose header into a valid stream state
                                    match purpose_header {
                                        StreamPurposeHeader::ConnectionManagement => {
                                            pending_removals += 1;
                                            stream_data.data = IncomingStreamData::ConnectionManagement(ConnectionManagementStream);
                                            continue 'iloop;
                                        },
                                        StreamPurposeHeader::StardustPayloads => {
                                            let channel_id = match reader.read_bytes(4).ok() {
                                                Some(val) => ChannelId::from(u32::from_be_bytes(TryInto::<[u8;4]>::try_into(val.as_slice_less_safe()).unwrap())),
                                                None => { todo!() },
                                            };
    
                                            pending_removals += 5;
                                            stream_data.data = IncomingStreamData::StardustPayloads(StardustPayloadsStream { id: channel_id });
                                            continue 'iloop
                                        },
                                    }
                                },
    
                                // Connection management stuff
                                IncomingStreamData::ConnectionManagement(_) => todo!(),
    
                                // Payload data
                                IncomingStreamData::StardustPayloads(_) => todo!(),
    
                                // Closed channel
                                IncomingStreamData::NeedsClosing(_) => todo!(),
                            }
                        }

                        // Remove a certain amount of bytes from the buffer
                        stream_data.buffer.remove_front(scratch, pending_removals);
                    },

                    // The stream is done sending.
                    Ok(None) => todo!(),

                    // No more bytes to read for now, but the stream isn't done sending.
                    Err(ReadError::Blocked) => todo!(),

                    // Stream reset
                    Err(ReadError::Reset(error_code)) => todo!(),
                }
            }
        }

        // Close and remove any streams queued for removal
        if remove_streams.len() != 0 {
            for stream_id in remove_streams.drain(..) {
                let reason = match active_recv_streams.get(&stream_id) {
                    Some(val) => {
                        match val.data {
                            IncomingStreamData::NeedsClosing(NeedsClosingStream { reason }) => reason.clone(),
                            _ => StreamErrorCode::NoReasonGiven,
                        }
                    },
                    None => { continue },
                };

                let _ = connection_inner
                    .recv_stream(stream_id)
                    .stop(reason.into());
                active_recv_streams.remove(&stream_id);
            }
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
            data: IncomingStreamData::PendingPurpose(PendingPurposeStream),
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
    PendingPurpose(PendingPurposeStream),
    ConnectionManagement(ConnectionManagementStream),
    StardustPayloads(StardustPayloadsStream),
    NeedsClosing(NeedsClosingStream),
}

struct PendingPurposeStream;

struct ConnectionManagementStream;

struct StardustPayloadsStream {
    id: ChannelId,
}

struct NeedsClosingStream {
    reason: StreamErrorCode,
}