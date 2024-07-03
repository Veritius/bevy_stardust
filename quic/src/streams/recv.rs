use std::collections::VecDeque;
use bytes::Bytes;
use super::*;
use crate::*;

pub(crate) struct Recv {
    queue: VecDeque<Bytes>,
}

impl StreamReader for Recv {
    fn read<S: ReadableStream>(&mut self, stream: &mut S, config: &QuicConfig) -> Result<usize, StreamReadError> {
        let mut read = 0;

        loop {
            match stream.read() {
                StreamReadOutcome::Chunk(chunk) => {
                    read += chunk.len();
                    self.queue.push_back(chunk);
                },

                StreamReadOutcome::Blocked => { break },

                StreamReadOutcome::Finished => { break },

                StreamReadOutcome::Error(err) => return Err(err),
            }
        }

        return Ok(read)
    }
}