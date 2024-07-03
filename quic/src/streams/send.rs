use std::collections::VecDeque;
use bytes::{Bytes, BytesMut};
use header::StreamHeader;
use quinn_proto::{VarInt, coding::Codec};
use super::*;

pub(crate) struct Send {
    framed: bool,
    queue: VecDeque<Bytes>,
}

impl Send {
    pub fn new(header: StreamHeader) -> Self {
        let mut queue = VecDeque::with_capacity(1);

        let framed = match header {
            StreamHeader::Stardust { channel: _ } => true,
        };

        let mut buffer = BytesMut::with_capacity(8);
        header.encode(&mut buffer);
        queue.push_back(buffer.freeze());

        return Self { framed, queue };
    }

    pub fn push(&mut self, chunk: Bytes) {
        if self.framed {
            let mut buffer = BytesMut::with_capacity(4);
            VarInt::from_u64(chunk.len() as u64).unwrap().encode(&mut buffer);
            self.queue.push_back(buffer.freeze());
        }

        self.queue.push_back(chunk);
    }
}

impl StreamWriter for Send {
    fn write<S: WritableStream>(&mut self, stream: &mut S) -> Result<usize, StreamWriteError> {
        let mut total = 0;

        while let Some(bytes) = self.queue.pop_front() {
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
                    self.queue.push_front(bytes);
                    continue;
                },

                // A block error means we must stop writing
                StreamWriteOutcome::Blocked => {
                    self.queue.push_front(bytes);
                    break;
                }

                // An error means the stream can no longer be written to
                StreamWriteOutcome::Error(err) => {
                    self.queue.push_front(bytes);
                    return Err(err)
                },
            }
        }

        return Ok(total);
    }
}