use std::collections::VecDeque;
use bytes::Bytes;

pub(crate) struct IncomingStream {
    queue: VecDeque<Bytes>,
}

impl IncomingStream {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub(super) fn push(&mut self, chunk: Bytes) {
        self.queue.push_back(chunk);
    }
}