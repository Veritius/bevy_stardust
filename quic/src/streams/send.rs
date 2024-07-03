use std::collections::VecDeque;
use bytes::Bytes;
use super::*;

pub(crate) struct Send {
    config: SendConfig,
    queue: VecDeque<Bytes>,
}

impl Send {
    pub fn new(config: SendConfig) -> Self {
        Self {
            config,
            queue: VecDeque::new(),
        }
    }

    #[inline]
    pub fn push(&mut self, chunk: Bytes) {
        todo!()
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

pub(crate) struct SendConfig {
    pub framed: bool,
}