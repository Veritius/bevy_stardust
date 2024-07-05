mod codes;
mod commitbuf;
mod header;
mod impls;
mod recv;
mod send;

pub(crate) use codes::ResetCode;
pub(crate) use header::StreamHeader;
pub(crate) use recv::{Recv, RecvOutput, StardustRecv};
pub(crate) use send::{Send, SendInit};

use bytes::Bytes;

/// A byte stream that can be written to.
pub(crate) trait WritableStream {
    fn write_to(&mut self, data: Bytes) -> StreamWriteOutcome;
}

/// A type that writes data to a stream.
pub(crate) trait StreamWriter {
    fn write<S: WritableStream>(&mut self, stream: &mut S) -> StreamWriteOutcome;
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
    Stopped(ResetCode),

    /// The stream was closed (finish or reset).
    Closed,
}

/// A byte stream that can be read from.
pub(crate) trait ReadableStream {
    fn read(&mut self) -> StreamReadOutcome;
}

/// A type that consumes data from a [`ReadableStream`] and handles it internally.
pub(crate) trait StreamReader {
    fn read_from<S: ReadableStream>(&mut self, stream: &mut S) -> Result<usize, StreamReadError>;
}

/// The outcome of reading from a stream.
pub(crate) enum StreamReadOutcome {
    /// A chunk of data was returned.
    Chunk(Bytes),

    /// The stream was blocked.
    /// Data may be returned in future calls.
    Blocked,

    /// The stream was finished.
    /// No further data will be returned.
    Finished,

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
    Reset(ResetCode),
}