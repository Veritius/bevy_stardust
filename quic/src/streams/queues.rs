use std::mem::swap as mem_swap;
use bevy::utils::smallvec::SmallVec;
use bytes::Bytes;
use super::*;

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