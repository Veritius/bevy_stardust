use std::collections::VecDeque;
use bytes::Bytes;
use super::{StreamTryWrite, StreamTryWriteOutcome};

pub(crate) struct OutgoingStreams {

}

impl OutgoingStreams {
    pub fn new() -> Self {
        Self {

        }
    }
}

struct WriteQueue(VecDeque<Bytes>);

impl WriteQueue {
    #[inline]
    pub fn push(&mut self, bytes: Bytes) {
        self.0.push_back(bytes);
    }

    pub fn write<S: StreamTryWrite>(&mut self, stream: &mut S) -> StreamTryWriteOutcome {
        while let Some(chunk) = self.0.pop_front() {
            match stream.try_write(chunk.clone()) {
                StreamTryWriteOutcome::Complete => { continue },

                StreamTryWriteOutcome::Partial(written) => {
                    self.0.push_front(chunk.slice(written..));
                    return StreamTryWriteOutcome::Partial(written);
                },

                StreamTryWriteOutcome::Blocked => {
                    self.0.push_front(chunk);
                    return StreamTryWriteOutcome::Blocked;
                }

                StreamTryWriteOutcome::Error(err) => return StreamTryWriteOutcome::Error(err),
            }
        }

        return StreamTryWriteOutcome::Complete;
    }
}