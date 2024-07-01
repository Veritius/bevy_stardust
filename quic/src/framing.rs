use std::cmp::Ordering;

use bevy::utils::smallvec::SmallVec;
use bytes::{Buf, Bytes, BytesMut};
use quinn_proto::{coding::Codec, VarInt, WriteError};
use crate::streams::{StreamWrite, StreamWriteOutcome};

pub(crate) struct FramedWriter {
    queue: SmallVec<[Bytes; 2]>,
}

impl FramedWriter {
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
                    swap.extend(drain);
                    break;
                }

                // An error means the stream can no longer be written to
                StreamWriteOutcome::Error(err) => {
                    swap.push(bytes);
                    swap.extend(drain);
                    return Err(err)
                },
            }
        }

        return Ok(total);
    }
}

pub(crate) struct FramedReader {
    queue: SmallVec<[Bytes; 2]>,
}

impl FramedReader {
    pub fn recv(&mut self, chunk: Bytes) {
        self.queue.push(chunk)
    }

    pub fn read(&mut self) -> Result<Bytes, ()> {
        todo!()
    }
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