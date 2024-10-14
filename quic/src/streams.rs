use std::{cmp::Ordering, collections::{BTreeMap, VecDeque}};
use bevy_stardust::prelude::*;
use bevy_stardust_extras::numbers::VarInt;
use bytes::{Buf, Bytes, BytesMut};
use crate::{segments::Segment, Connection, ConnectionEvent, ResetCode};

/// An event used by the state machine to control QUIC streams.
pub enum StreamEvent {
    /// Open a new stream.
    /// 
    /// This is always sent before `Transmit`.
    Open {
        /// The stream that is opened.
        id: SendStreamId,
    },

    /// Send a chunk of data over a stream.
    /// 
    /// Only occurs after an `Open` event with the same `id` is sent.
    Transmit {
        /// The stream to send over.
        id: SendStreamId,

        /// The chunk of data to send.
        chunk: Bytes,
    },

    /// Set the priority of a stream.
    SetPriority {
        /// The stream which should have its priority changed.
        id: SendStreamId,

        /// The priority value.
        priority: u32,
    },

    /// Reset a stream.
    Reset {
        /// The stream to reset.
        id: SendStreamId,

        /// The reset code.
        code: ResetCode,
    },

    /// Finish a stream.
    Finish {
        /// The stream to finish.
        id: SendStreamId,
    },

    /// Stop a stream.
    Stop {
        /// The stream to stop.
        id: RecvStreamId,

        /// The reset code.
        code: ResetCode,
    },
}

/// A stream identifier for an **outgoing** (sending) QUIC stream.
/// 
/// Generated by the state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SendStreamId(pub u64);

/// A stream identifier for an **incoming** (receiving) QUIC stream.
/// 
/// Generated by the QUIC implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecvStreamId(pub u64);

pub(crate) struct OutgoingStreams {
    unique_id_index: u64,
    channel_ids: BTreeMap<ChannelId, SendStreamId>,
    channel_ids_reverse: BTreeMap<SendStreamId, ChannelId>,
}

impl OutgoingStreams {
    const SEND_STREAM_ID_LIMIT: u64 = 2u64.pow(62) - 1;

    pub fn new() -> Self {
        Self {
            unique_id_index: 0,
            channel_ids: BTreeMap::new(),
            channel_ids_reverse: BTreeMap::new(),
        }
    }
}

impl Connection {
    /// Call when a stream is stopped.
    pub fn stream_stopped(&mut self, stream: SendStreamId) {
        if let Some(id) = self.outgoing_streams.channel_ids_reverse.remove(&stream) {
            self.outgoing_streams.channel_ids.remove(&id);
        }
    }
}

impl Connection {
    pub(crate) fn stream_segment_transient(
        &mut self,
        segment: Segment,
    ) {
        let id = self.open_stream_inner();
        self.stream_segment_existing(segment, id);
        self.stream_event(StreamEvent::Finish { id });
    }

    pub(crate) fn stream_segment_existing(
        &mut self,
        segment: Segment,
        id: SendStreamId,
    ) {
        // Add the length prefix for the segment
        let size = segment.size() as u32;
        let mut buffer = BytesMut::with_capacity(8);
        VarInt::from_u32(size).write(&mut buffer).unwrap();

        self.stream_event(StreamEvent::Transmit {
            id,
            chunk: buffer.freeze(),
        });

        self.stream_event(StreamEvent::Transmit {
            id,
            chunk: segment.header.alloc(),
        });

        self.stream_event(StreamEvent::Transmit {
            id,
            chunk: segment.payload.clone(),
        });
    }

    pub(crate) fn stream_segment_existing_iter(
        &mut self,
        id: SendStreamId,
        iter: impl Iterator<Item = Segment>,
    ) {
        for segment in iter {
            self.stream_segment_existing(segment, id);
        }
    }

    pub(crate) fn get_channel_stream(
        &mut self,
        channel: ChannelId
    ) -> SendStreamId {
        match self.outgoing_streams.channel_ids.get(&channel) {
            Some(id) => return *id,
            None => {
                let id = self.open_stream_inner();
                self.outgoing_streams.channel_ids.insert(channel, id);
                self.outgoing_streams.channel_ids_reverse.insert(id, channel);
                return id;
            },
        }
    }

    fn open_stream_inner(
        &mut self,
    ) -> SendStreamId {
        let index = self.outgoing_streams.unique_id_index;
        assert!(index < OutgoingStreams::SEND_STREAM_ID_LIMIT, "Exceeded send ID limit");
        self.outgoing_streams.unique_id_index += 1;
        let id = SendStreamId(index);
        self.stream_event(StreamEvent::Open { id });
        return id;
    }

    fn stream_event(
        &mut self,
        event: StreamEvent,
    ) {
        self.shared.events.push(ConnectionEvent::StreamEvent(event));
    }
}

pub(crate) struct IncomingStreams {
    buffers: BTreeMap<RecvStreamId, StreamChunkBuf>,
}

impl IncomingStreams {
    pub fn new() -> Self {
        Self {
            buffers: BTreeMap::new(),
        }
    }
}

impl Connection {
    /// Call when a chunk of data is received on a stream.
    pub fn stream_recv(&mut self, id: RecvStreamId, chunk: Bytes) {
        // Get or create a buffer for the new receive stream
        let buffer = self.incoming_streams.buffers.entry(id)
            .or_insert_with(|| StreamChunkBuf::new());

        // Push the chunk to the buffer
        buffer.push(chunk);

        // Read the length without committing ourselves
        let mut view = buffer.view();
        let len = match VarInt::read(&mut view) {
            Ok(val) => Into::<u64>::into(val) as usize,
            Err(_) => { return }, // not enough data, stop reading
        };

        // Check there's actually anything to read
        let remaining = view.remaining();
        if remaining < len { return }
        view.commit(); // Commit to reading the rest
        let original = buffer.remaining();

        let segment = match Segment::parse(buffer) {
            Ok(segment) => segment,
            Err(_) => {
                // Since it failed, we likely didn't read all of it
                // We advance the buffer to get rid of the wrongly-parsed segment
                let diff = original - remaining;
                buffer.advance(diff);

                // If these numbers differ, something has gone wrong
                debug_assert_eq!(original - remaining, buffer.remaining());

                // We're done
                return;
            },
        };

        // If these numbers differ, something has gone wrong
        debug_assert_eq!(original - remaining, buffer.remaining());

        // Handle the incoming segment
        self.recv_segment(segment);
    }

    /// Call when an incoming stream is reset.
    pub fn stream_reset(&mut self, id: RecvStreamId) {
        self.incoming_streams.buffers.remove(&id);
    }

    /// Call when an incoming stream is finished.
    pub fn stream_finished(&mut self, id: RecvStreamId) {
        self.incoming_streams.buffers.remove(&id);
    }
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

    fn view(&mut self) -> StreamChunkView {
        StreamChunkView {
            buf: self,
            index: 0,
            cursor: 0,
        }
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

struct StreamChunkView<'a> {
    buf: &'a mut StreamChunkBuf,
    index: usize,
    cursor: usize,
}

impl<'a> StreamChunkView<'a> {
    fn commit(self) {
        let sum: usize = (0..self.index)
            .into_iter()
            .map(|v| self.buf.queue[v].len())
            .sum();

        let sum = self.cursor + sum;
        self.buf.advance(sum);
    }
}

impl<'a> Buf for StreamChunkView<'a> {
    fn remaining(&self) -> usize {
        let mut chunks = self.buf.queue.iter()
            .skip(self.index)
            .map(|v| v.len());

        let mut sum = 0;

        if let Some(next) = chunks.next() {
            sum += next - self.cursor;
        }

        chunks.for_each(|v| sum += v);

        return sum;
    }

    fn chunk(&self) -> &[u8] {
        &self.buf.queue[0]
    }

    fn advance(&mut self, mut cnt: usize) {
        let rem = self.buf.queue[self.index].len() - self.cursor;

        match rem.cmp(&cnt) {
            Ordering::Less | Ordering::Equal => {
                self.cursor = 0;
                self.index += 1;
                cnt -= rem;
            },

            Ordering::Greater => {
                self.cursor += cnt;
                return;
            },
        }

        while cnt > 0 {
            let len = self.buf.queue[self.index].len();
            match len.cmp(&cnt) {
                Ordering::Less | Ordering::Equal => {
                    self.index += 1;
                    cnt -= len;
                },

                Ordering::Greater => {
                    self.cursor += cnt;
                    return;
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