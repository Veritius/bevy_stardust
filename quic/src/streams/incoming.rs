use std::collections::{BTreeMap, VecDeque};
use bevy_stardust::prelude::*;
use bytes::Bytes;
use crate::Connection;
use super::RecvStreamId;

impl Connection {
    /// Call when a chunk of data is received on a stream.
    pub fn stream_recv(&mut self, id: RecvStreamId, chunk: Bytes) {
        todo!()
    }

    /// Call when a new incoming stream is opened.
    pub fn stream_opened(&mut self, id: RecvStreamId) {
        self.incoming_streams.recv_stream_opened(id)
    }
    
    /// Call when an incoming stream is reset.
    pub fn stream_reset(&mut self, id: RecvStreamId) {
        self.incoming_streams.recv_stream_reset(id);
    }
    
    /// Call when an incoming stream is finished.
    pub fn stream_finished(&mut self, id: RecvStreamId) {
        self.incoming_streams.recv_stream_finished(id);
    }
}

pub(crate) struct IncomingStreams {
    streams: BTreeMap<RecvStreamId, IncomingStream>,
}

impl IncomingStreams {
    pub fn new() -> Self {
        Self {
            streams: BTreeMap::new(),
        }
    }

    pub fn recv_stream_opened(&mut self, id: RecvStreamId) {
        self.streams.insert(id, IncomingStream::new());
    }

    pub fn recv_stream_reset(&mut self, id: RecvStreamId) {
        self.streams.remove(&id);
    }

    pub fn recv_stream_finished(&mut self, id: RecvStreamId) {
        self.streams.remove(&id);
    }
}

struct IncomingStream {
    mode: Option<IncomingStreamMode>,
    queue: VecDeque<Bytes>,
}

impl IncomingStream {
    fn new() -> Self {
        Self {
            mode: None,
            queue: VecDeque::new(),
        }
    }

    fn mode(&self) -> Option<IncomingStreamMode> {
        match self.mode {
            None => {
                todo!()
            },

            v => v,
        }
    }

    fn push(&mut self, chunk: Bytes) {
        self.queue.push_back(chunk);
    }

    fn pull(&mut self) -> IncomingStreamChunkIter {
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