use std::collections::VecDeque;
use bytes::Bytes;
use quinn_proto::{SendStream, WriteError};

pub(crate) struct StreamWriteQueue {
    queue: VecDeque<Bytes>,
}

impl StreamWriteQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn push(&mut self, chunk: Bytes) {
        self.queue.push_back(chunk);
    }

    pub fn write(&mut self, stream: &mut SendStream) -> Result<bool, WriteError> {
        // Pop from queue
        while let Some(chunk) = self.queue.pop_front() {
            // Try to write
            match stream.write(&chunk[..]) {
                // Chunk was fully written
                Ok(l) if l == chunk.len() => {
                    // Return whether or not the queue is drained
                    let drained = self.queue.len() == 0;
                    return Ok(drained);
                },

                // Chunk was partially written
                Ok(l) => {
                    // Remove the written portion and put it back
                    let slice = chunk.slice(l..);
                    self.queue.push_front(slice);

                    // Partial writes mean the queue is not drained
                        return Ok(false);
                    }

                    // No writing was possible due to congestion
                    Err(WriteError::Blocked) => {
                        // Put the item back into the queue
                        self.queue.push_front(chunk);
                        return Ok(false);
                    }
        
                // Error while writing
                Err(err) => return Err(err),
            }
        }

        // Queue is drained
        return Ok(true);
    }
}