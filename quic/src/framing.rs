use std::mem::swap as mem_swap;
use std::cmp::Ordering;
use bevy::utils::smallvec::SmallVec;
use bytes::{Buf, Bytes, BytesMut};
use quinn_proto::{coding::Codec, VarInt, WriteError};
use crate::{streams::{StreamWrite, StreamWriteOutcome}, QuicConfig};

pub(crate) struct FramedWriter {
    queue: SmallVec<[Bytes; 2]>,
}

impl FramedWriter {
    pub fn new() -> Self {
        Self {
            queue: SmallVec::new(),
        }
    }

    pub fn queue(&mut self, message: Bytes) {
        // Framed message length header
        let mut buf = BytesMut::with_capacity(4);
        VarInt::from_u64(message.len() as u64).unwrap().encode(&mut buf);

        // Append to queue
        self.queue.push(buf.freeze());
        self.queue.push(message);
    }

    pub fn write<S>(&mut self, stream: &mut S) -> Result<usize, WriteError>
    where
        S: StreamWrite,
    {
        let mut total = 0;
        let mut swap: SmallVec<[Bytes; 2]> = SmallVec::with_capacity(self.queue.len());
        let mut drain = self.queue.drain(..);

        while let Some(bytes) = drain.next() {
            match stream.write(&bytes[..]) {
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
                StreamWriteOutcome::Error(WriteError::Blocked) => {
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

pub(crate) struct FramedReader {
    queue: SmallVec<[Bytes; 2]>,
}

impl FramedReader {
    pub fn new() -> Self {
        Self {
            queue: SmallVec::new(),
        }
    }

    #[inline]
    pub fn push(&mut self, chunk: Bytes) {
        self.queue.push(chunk)
    }

    pub fn read(&mut self, config: &QuicConfig) -> FramedReaderOutcome {
        // Create a non-contiguous Buf implementor for the queue
        let mut reader = QueueBuf::new(&self.queue);

        // Decode the length of the reader
        let length = match VarInt::decode(&mut reader) {
            Ok(v) => v.into_inner() as usize,
            Err(_) => return FramedReaderOutcome::Waiting,
        };

        // Prevents a denial of service attack
        if length > config.maximum_framed_message_length {
            return FramedReaderOutcome::Error;
        }

        // Check if the reader has the full message
        if reader.remaining() < length {
            return FramedReaderOutcome::Waiting;
        }

        // Copy the message to its own contiguous allocation
        // This is necessary since the reader is non-contiguous
        let payload = reader.copy_to_bytes(length);

        // Variables for the commit operation
        let mut consumed = reader.consumed();
        let mut swap: SmallVec<[Bytes; 2]> = SmallVec::with_capacity(self.queue.len());
        let mut drain = self.queue.drain(..);

        // 'Commit', ensuring we don't read the same data twice
        while consumed > 0 {
            let f = drain.next().unwrap();

            if consumed >= f.len() {
                consumed -= f.len(); 
                continue;
            }

            swap.push(f.slice(consumed..));
            consumed -= f.len();
        }

        // Return the queue back
        swap.extend(drain);
        mem_swap(&mut self.queue, &mut swap);

        // Return the message (success)
        return FramedReaderOutcome::Message(payload);
    }
}

pub(crate) enum FramedReaderOutcome {
    Message(Bytes),
    Waiting,
    Error,
}

struct QueueBuf<'a> {
    remaining: usize,
    cursor: usize,
    index: usize,
    inner: &'a [Bytes],
}

impl<'a> QueueBuf<'a> {
    fn new(inner: &'a [Bytes]) -> QueueBuf<'a> {
        Self {
            remaining: inner.iter().map(|v| v.len()).sum(),
            cursor: 0,
            index: 0,
            inner,
        }
    }

    fn consumed(&self) -> usize {
        let initial: usize = self.inner.iter().map(|v| v.len()).sum();
        return initial - self.remaining;
    }
}

impl<'a> Buf for QueueBuf<'a> {
    #[inline]
    fn remaining(&self) -> usize {
        self.remaining
    }

    #[inline]
    fn chunk(&self) -> &'a [u8] {
        &self.inner[self.index][self.cursor..]
    }

    fn advance(&mut self, cnt: usize) {
        if cnt > self.remaining { panic!("Overran buffer"); }
        self.remaining -= cnt;

        let sel = &self.inner[self.index];
        match (self.cursor + cnt).cmp(&sel.len()) {
            Ordering::Less => {
                self.cursor += cnt;
            },

            Ordering::Equal => {
                self.cursor = 0;
                self.index += 1;
            },

            Ordering::Greater => {
                self.cursor = cnt - sel.len();
                self.index += 1;
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reader_limit_test() {
        // Very generic config
        const CONFIG: QuicConfig = QuicConfig {
            maximum_transport_units: usize::MAX,
            maximum_framed_message_length: 64,
            maximum_buffered_frame_data: usize::MAX,
        };

        // Create a stream with an intentionally wrong length prefix
        let mut stream = BytesMut::with_capacity(128);
        VarInt::from_u32(1024).encode(&mut stream);
        stream.extend_from_slice(b"Hello, world!");
        let stream = stream.freeze();

        // Create a new reader
        let mut reader = FramedReader::new();
        reader.push(stream);

        // The reader should recognise the prefix is too long
        match reader.read(&CONFIG) {
            FramedReaderOutcome::Error => {},
            _ => panic!(),
        }
    }

    #[test]
    fn write_read_test() {
        // Very generic config
        const CONFIG: QuicConfig = QuicConfig {
            maximum_transport_units: usize::MAX,
            maximum_framed_message_length: usize::MAX,
            maximum_buffered_frame_data: usize::MAX,
        };

        // The message we'll be using
        const MESSAGE: &[u8] = b"Hello, world!";

        // Create a new reader
        let mut reader = FramedReader::new();

        // Nothing in the queue
        match reader.read(&CONFIG) {
            FramedReaderOutcome::Waiting => {},
            _ => panic!(),
        }

        // Write a new message to a stream for testing
        let mut stream = BytesMut::with_capacity(128);
        let mut writer = FramedWriter::new();
        writer.queue(Bytes::from_static(&MESSAGE));
        writer.write(&mut stream).unwrap();
        let stream = stream.freeze();

        // Simplest case for the reader
        reader.push(stream.clone());
        match reader.read(&CONFIG) {
            FramedReaderOutcome::Message(message) => assert_eq!(&message[..], MESSAGE),
            _ => panic!(),
        }

        // The frame is broken into two chunks but fully present
        reader.push(stream.slice(..6));
        reader.push(stream.slice(6..));
        match reader.read(&CONFIG) {
            FramedReaderOutcome::Message(message) => assert_eq!(&message[..], MESSAGE),
            _ => panic!(),
        }

        // The frame is not yet fully present
        reader.push(stream.slice(..8));
        match reader.read(&CONFIG) {
            FramedReaderOutcome::Waiting => {},
            _ => panic!(),
        }

        // The frame is now fully present
        reader.push(stream.slice(8..));
        match reader.read(&CONFIG) {
            FramedReaderOutcome::Message(message) => assert_eq!(&message[..], MESSAGE),
            _ => panic!(),
        }

        // An extremely fragmented frame
        // This is the worst case scenario
        reader.push(stream.slice(0..1));
        reader.push(stream.slice(1..3));
        reader.push(stream.slice(3..4));
        reader.push(stream.slice(4..7));
        reader.push(stream.slice(7..12));
        reader.push(stream.slice(12..));
        match reader.read(&CONFIG) {
            FramedReaderOutcome::Message(message) => assert_eq!(&message[..], MESSAGE),
            _ => panic!(),
        }
    }
}