use std::collections::VecDeque;
use bevy_stardust::prelude::*;
use bytes::Bytes;
use crate::Connection;
use super::RecvStreamId;

impl Connection {
    /// Call when a chunk of data is received on a stream.
    pub fn stream_recv(&mut self, stream: RecvStreamId, chunk: Bytes) {
        if !self.incoming_streams.contains_key(&stream) {
            self.stream_open(stream);
        }

        let stream = self.incoming_streams.get_mut(&stream).unwrap();
        stream.push(chunk);

        todo!()
    }
}

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