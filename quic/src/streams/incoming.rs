use std::collections::VecDeque;
use bevy_stardust::prelude::ChannelId;
use bytes::Bytes;

pub(crate) struct IncomingStream {
    mode: Option<IncomingStreamMode>,
    queue: VecDeque<Bytes>,
}

impl IncomingStream {
    pub fn new() -> Self {
        Self {
            mode: None,
            queue: VecDeque::new(),
        }
    }

    pub(super) fn mode(&self) -> Option<IncomingStreamMode> {
        match self.mode {
            None => {
                todo!()
            },

            v => v,
        }
    }

    pub(super) fn push(&mut self, chunk: Bytes) {
        self.queue.push_back(chunk);
    }

    pub(super) fn pull(&mut self) -> IncomingStreamChunkIter {
        IncomingStreamChunkIter { stream: self }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IncomingStreamMode {
    Stardust {
        channel: ChannelId,
    },

    WrappedDatagram,
}

pub(super) struct IncomingStreamChunkIter<'a> {
    stream: &'a mut IncomingStream,
}

impl Iterator for IncomingStreamChunkIter<'_> {
    type Item = Result<Bytes, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}