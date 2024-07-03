mod queues;

pub(crate) use queues::ChunkQueueWriter;

use bytes::{BufMut, Bytes, BytesMut};
use quinn_proto::{Chunks, SendStream, WriteError};
use crate::QuicConfig;

// #############################
// ##### Traits and Errors #####
// #############################

/// A byte stream that can be written to.
pub(crate) trait WritableStream {
    fn write(&mut self, data: Bytes) -> StreamWriteOutcome;
}

/// The outcome of attempting to write to the stream.
pub(crate) enum StreamWriteOutcome {
    /// The full slice was written.
    Complete,

    /// Only a part of the slice was written.
    /// This includes the amount that was written.
    Partial(usize),

    /// The stream is blocked and cannot be written to.
    Blocked,

    /// The stream encountered an error.
    /// This variant is fatal and means the stream is unrecoverable.
    Error(StreamWriteError),
}

/// An error encountered while writing to a stream.
#[derive(Debug, Clone, Copy)]
pub(crate) enum StreamWriteError {
    /// The stream was stopped.
    Stopped(u64),

    /// The stream was closed (finish or reset).
    Closed,
}

/// A type that writes data to a stream.
pub(crate) trait StreamWriter {
    fn write<S: WritableStream>(&mut self, stream: &mut S) -> Result<usize, StreamWriteError>;
}

/// A byte stream that can be read from.
pub(crate) trait ReadableStream {
    fn read(&mut self) -> StreamReadOutcome;
}

/// The outcome of reading from a stream.
pub(crate) enum StreamReadOutcome {
    /// A chunk of data was returned.
    Chunk(Bytes),

    /// The stream was finished.
    /// No further data will be returned.
    Finished,

    /// The stream was blocked.
    /// Data may be returned in future calls.
    Blocked,

    // An error was encountered while reading.
    // This variant is fatal and means the stream is unrecoverable.
    Error(StreamReadError),
}

/// An error encountered when trying to read from a stream.
#[derive(Debug, Clone, Copy)]
pub(crate) enum StreamReadError {
    /// The stream was closed.
    Closed,

    /// The stream was reset.
    Reset(u64),
}

/// A type that consumes data from a [`ReadableStream`] and handles it internally.
pub(crate) trait StreamReader {
    fn read<S: ReadableStream>(&mut self, stream: &mut S, config: &QuicConfig) -> Result<usize, StreamWriteError>;
}

// ###########################
// ##### Implementations #####
// ###########################

impl WritableStream for BytesMut {
    fn write(&mut self, data: Bytes) -> StreamWriteOutcome {
        self.put(data);
        StreamWriteOutcome::Complete
    }
}

impl WritableStream for SendStream<'_> {
    fn write(&mut self, data: Bytes) -> StreamWriteOutcome {
        match self.write_chunks(&mut [data.clone()]) {
            Ok(written) if written.bytes == data.len() => StreamWriteOutcome::Complete,

            Ok(written) => StreamWriteOutcome::Partial(written.bytes),

            Err(WriteError::Blocked) => StreamWriteOutcome::Blocked,

            Err(err) => StreamWriteOutcome::Error(match err {
                WriteError::Stopped(code) => StreamWriteError::Stopped(code.into_inner()),
                WriteError::ClosedStream => StreamWriteError::Closed,
                WriteError::Blocked => unreachable!(),
            }),
        }
    }
}

impl ReadableStream for Bytes {
    fn read(&mut self) -> StreamReadOutcome {
        if self.len() == 0 { return StreamReadOutcome::Blocked }
        let cloned = self.clone();
        *self = self.slice(self.len()..);
        return StreamReadOutcome::Chunk(cloned)
    }
}

impl ReadableStream for Chunks<'_> {
    fn read(&mut self) -> StreamReadOutcome {
        match self.next(usize::MAX) {
            Ok(Some(chunk)) => StreamReadOutcome::Chunk(chunk.bytes),

            Ok(None) => StreamReadOutcome::Finished,

            Err(error) => match error {
                quinn_proto::ReadError::Blocked => StreamReadOutcome::Blocked,
                quinn_proto::ReadError::Reset(code) => StreamReadOutcome::Error(StreamReadError::Reset(code.into_inner())),
            },
        }
    }
}