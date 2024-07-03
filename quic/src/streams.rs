use std::mem::swap as mem_swap;
use bevy::utils::smallvec::SmallVec;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use quinn_proto::{coding::Codec, Chunks, SendStream, VarInt, WriteError};
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

pub(crate) struct ChunkQueueWriter {
    queue: SmallVec<[Bytes; 2]>,
}

impl ChunkQueueWriter {
    pub fn new() -> Self {
        Self {
            queue: SmallVec::new(),
        }
    }

    #[inline]
    pub fn queue(&mut self, message: Bytes) {
        self.queue.push(message);
    }
}

impl StreamWriter for ChunkQueueWriter {
    fn write<S>(&mut self, stream: &mut S) -> Result<usize, StreamWriteError>
    where
        S: WritableStream,
    {
        let mut total = 0;
        let mut swap: SmallVec<[Bytes; 2]> = SmallVec::with_capacity(self.queue.len());
        let mut drain = self.queue.drain(..);

        while let Some(bytes) = drain.next() {
            match stream.write(bytes.clone()) {
                // A complete write means we can try again
                StreamWriteOutcome::Complete => {
                    total += bytes.len();
                    continue;
                },

                // A partial write means we have to stop
                StreamWriteOutcome::Partial(written) => {
                    total += written;
                    let bytes = bytes.slice(written..);
                    swap.push(bytes);
                    continue;
                },

                // A block error means we must stop writing
                StreamWriteOutcome::Blocked => {
                    swap.push(bytes);
                    break;
                }

                // An error means the stream can no longer be written to
                StreamWriteOutcome::Error(err) => {
                    swap.push(bytes);
                    return Err(err)
                },
            }
        }

        swap.extend(drain);
        mem_swap(&mut self.queue, &mut swap);
        return Ok(total);
    }
}

pub(crate) enum StreamOpenHeader {
    StardustPersistent {
        channel: u32,
    },

    StardustTransient {
        channel: u32,
    },
}

impl StreamOpenHeader {
    pub fn decode<B: Buf>(buf: &mut B) -> Result<Self, ()> {
        let ident = VarInt::decode(buf).map_err(|_| ())?.into_inner();

        fn decode<B: Buf, T: TryFrom<u64>>(buf: &mut B) -> Result<u32, ()> {
            let varint = VarInt::decode(buf).map_err(|_| ())?.into_inner();
            let value = u32::try_from(varint).map_err(|_| ())?;
            return Ok(value);
        }

        match ident {
            0 => Ok(StreamOpenHeader::StardustPersistent {
                channel: decode::<B, u32>(buf)?,
            }),

            1 => Ok(Self::StardustTransient {
                channel: decode::<B, u32>(buf)?,
            }),

            _ => Err(()),
        }
    }

    pub fn encode<B: BufMut>(&self, buf: &mut B) {
        match self {
            StreamOpenHeader::StardustPersistent { channel } => {
                VarInt::from_u32(0).encode(buf);
                VarInt::from_u32(*channel).encode(buf);
            },

            StreamOpenHeader::StardustTransient { channel } => {
                VarInt::from_u32(1).encode(buf);
                VarInt::from_u32(*channel).encode(buf);
            },
        }
    }
}