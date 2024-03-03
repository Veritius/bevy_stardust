use std::collections::HashMap;
use bevy_ecs::prelude::*;
use bevy_stardust::{channels::registry::ChannelRegistry, connections::peer::NetworkPeer};
use quinn_proto::{Dir, ReadError, ReadableError, StreamId, Streams, VarInt};
use untrusted::{Input, Reader};
use crate::{streams::StreamErrorCode, QuicConnection};

mod pending; use pending::*;

pub(super) fn read_messages_from_streams_system(
    mut connections: Query<(Entity, &mut QuicConnection), With<NetworkPeer>>,
    registry: Res<ChannelRegistry>,
) {
    // Any processing that can run in parallel runs here
    connections.par_iter_mut().for_each(|(entity, mut connection)| {
        // Scratch space for some operations
        let mut scratch: Vec<u8> = Vec::with_capacity(512); // 512 seems like a reasonable default
        let scratch = &mut scratch;

        // Split borrow function to help out borrowck
        #[inline]
        fn split_borrow_stream_ctrl(conn: &mut QuicConnection) -> (
            Streams,
            &mut HashMap<StreamId, IncomingStream>
        ) {(
            conn.inner.get_mut().streams(), &mut conn.recv_streams
        )}

        // Borrow many fields using the split borrow function
        let (
            mut streams,
            recv_streams,
        ) = split_borrow_stream_ctrl(&mut connection);

        // Accept all new unidirectional streams
        while let Some(stream_id) = streams.accept(Dir::Uni) {
            recv_streams.insert(stream_id, IncomingStream::default());
        }

        // Accept all new bidirectional streams
        while let Some(stream_id) = streams.accept(Dir::Bi) {
            recv_streams.insert(stream_id, IncomingStream::default());
        }

        // Same as before
        #[inline]
        fn split_borrow(connection: &mut QuicConnection) -> (
            &mut quinn_proto::Connection,
            &mut HashMap<quinn_proto::StreamId, IncomingStream>,
        ) {(
            connection.inner.get_mut(),
            &mut connection.recv_streams
        )}

        // Still the same as before
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

            // Context object for chunk reading
            let context = IncomingStreamProcessingContext {
                stream_id: stream_id.clone(),
                registry: &registry,
            };

            // Read all available chunks
            loop {
                let action = match chunks.next(usize::MAX) {
                    Ok(Some(chunk)) => {
                        // Tracker for reading chunks
                        stream_data.buffer.insert_back(&chunk.bytes);
                        let mut reader = Reader::new(Input::from(stream_data.buffer.read_slice()));
                        let mut consumed = 0;

                        // Construct payload variant
                        let chunk = UnprocessedChunk::Payload(TrackingReader {
                            consumed: &mut consumed,
                            reader: &mut reader,
                        });

                        // Pass off processing to the data object
                        stream_data.data.process_chunk(&context, chunk)
                    },

                    Ok(None) => {
                        // Construct finish variant
                        let chunk = UnprocessedChunk::Finished;

                        // Pass off processing to the data object
                        stream_data.data.process_chunk(&context, chunk)
                    },

                    Err(ReadError::Reset(reason)) => {
                        // Construct reset variant
                        let chunk = UnprocessedChunk::Reset(reason);

                        // Pass off processing to the data object
                        stream_data.data.process_chunk(&context, chunk)
                    }

                    // We've run out of chunks to read for this stream
                    Err(ReadError::Blocked) => break,
                };

                // Apply the actions that our object has said to do
                match action {
                    ProcessingOutputAction::DoNothing => todo!(),
                    ProcessingOutputAction::CloseStream(_) => todo!(),
                    ProcessingOutputAction::ReplaceSelf(_) => todo!(),
                    ProcessingOutputAction::Multiple(_) => todo!(),
                }
            }
        }
    });
}

pub(crate) struct IncomingStream {
    buffer: IncomingStreamBuffer,
    data: Box<dyn IncomingStreamData>,
}

impl Default for IncomingStream {
    fn default() -> Self {
        Self {
            buffer: IncomingStreamBuffer(Vec::with_capacity(32)),
            data: Box::from(pending::PendingStreamData),
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

enum UnprocessedChunk<'a> {
    Payload(TrackingReader<'a>),
    Finished,
    Reset(VarInt),
}

struct TrackingReader<'a> {
    consumed: &'a mut usize,
    reader: &'a mut Reader<'a>,
}

impl TrackingReader<'_> {
    fn commit_bytes(&mut self, amount: usize) {
        *self.consumed += amount;
    }
}

impl<'a> std::ops::Deref for TrackingReader<'a> {
    type Target = Reader<'a>;

    fn deref(&self) -> &Self::Target {
        self.reader
    }
}

impl std::ops::DerefMut for TrackingReader<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.reader
    }
}

struct IncomingStreamProcessingContext<'a> {
    stream_id: StreamId,
    registry: &'a ChannelRegistry,
}

trait IncomingStreamData: Send + Sync {
    fn process_chunk(
        &mut self,
        context: &IncomingStreamProcessingContext,
        chunk: UnprocessedChunk,
    ) -> ProcessingOutputAction;
}

enum ProcessingOutputAction {
    DoNothing,
    CloseStream(StreamErrorCode),
    ReplaceSelf(Box<dyn IncomingStreamData>),
    Multiple(Vec<ProcessingOutputAction>),
}

impl From<StreamErrorCode> for ProcessingOutputAction {
    fn from(value: StreamErrorCode) -> Self {
        Self::CloseStream(value)
    }
}