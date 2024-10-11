use std::{cmp::Ordering, collections::{BTreeMap, VecDeque}};
use bevy_stardust::prelude::*;
use bytes::{Buf, Bytes};
use crate::Connection;
use super::RecvStreamId;

impl Connection {
    /// Call when a chunk of data is received on a stream.
    pub fn stream_recv(&mut self, id: RecvStreamId, chunk: Bytes) {
        let stream = self.incoming_streams.get_or_open_stream(id);
        stream.push(chunk);

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

    fn get_or_open_stream(&mut self, id: RecvStreamId) -> &mut IncomingStream {
        self.streams.entry(id).or_insert_with(|| IncomingStream::new())
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
    queue: StreamChunkBuf,
}

impl IncomingStream {
    fn new() -> Self {
        Self {
            mode: None,
            queue: StreamChunkBuf::new(),
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

    #[inline]
    fn push(&mut self, chunk: Bytes) {
        self.queue.push(chunk);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IncomingStreamMode {
    Stardust {
        channel: ChannelId,
    },

    WrappedDatagram,
}

struct StreamChunkBuf {
    queue: VecDeque<Bytes>,
}

impl StreamChunkBuf {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    #[inline]
    fn push(&mut self, chunk: Bytes) {
        self.queue.push_back(chunk);
    }
}

impl Buf for StreamChunkBuf {
    fn remaining(&self) -> usize {
        self.queue.iter().map(|b| b.len()).sum()
    }

    fn chunk(&self) -> &[u8] {
        &self.queue[0]
    }

    fn advance(&mut self, mut cnt: usize) {
        while cnt > 0 {
            let front = &mut self.queue[0];

            match front.len().cmp(&cnt) {
                Ordering::Less | Ordering::Equal => {
                    cnt -= front.len();
                    self.queue.pop_front();
                },

                Ordering::Greater => {
                    *front = front.slice(cnt..);
                    cnt = 0;
                },
            }
        }
    }
}

#[test]
fn stream_chunk_buf_test() {
    let mut buf = StreamChunkBuf::new();

    buf.push(Bytes::from_static(b"Hello,")); // 6
    buf.push(Bytes::from_static(b"")); // 0
    buf.push(Bytes::from_static(b" ")); // 1
    buf.push(Bytes::from_static(b"world!")); // 6

    assert_eq!(buf.remaining(), 13);

    buf.advance(3);

    assert_eq!(buf.remaining(), 10);

    buf.advance(5);

    assert_eq!(buf.remaining(), 5);

    buf.advance(3);

    assert_eq!(buf.remaining(), 2);
}