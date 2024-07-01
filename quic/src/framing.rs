use bevy::utils::smallvec::SmallVec;
use bytes::{Bytes, BytesMut};
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