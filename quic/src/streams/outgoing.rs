use std::collections::VecDeque;
use bevy::utils::HashMap;
use bevy_stardust::messages::ChannelId;
use bevy_stardust_extras::numbers::VarInt;
use bytes::{Bytes, BytesMut};
use super::{header::StreamPurpose, StreamId, StreamTryWrite, StreamTryWriteOutcome};

pub(crate) struct OutgoingStreams {
    channels: HashMap<ChannelId, StreamId>,
    streams: HashMap<StreamId, OutgoingStreamState>,
}

impl OutgoingStreams {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            streams: HashMap::new(),
        }
    }

    pub fn open(&mut self, stream: StreamId, purpose: StreamPurpose, transient: bool) {
        self.streams.insert(stream, OutgoingStreamState {
            queue: WriteQueue::new(),
        });
    }

    pub fn get(&mut self, stream: StreamId) -> Option<OutgoingStream> {
        let state = self.streams.get_mut(&stream)?;
        return Some(OutgoingStream { state });
    }

    #[must_use]
    pub fn open_and_get(&mut self, stream: StreamId, purpose: StreamPurpose, transient: bool) -> OutgoingStream {
        self.open(stream, purpose, transient);
        return self.get(stream).unwrap();
    }

    pub fn write<S: StreamTryWrite>(&mut self, id: StreamId, stream: &mut S) -> Option<StreamTryWriteOutcome> {
        let outgoing = self.streams.get_mut(&id)?;
        return outgoing.queue.write(stream);
    }
}

pub(crate) struct OutgoingStream<'a> {
    state: &'a mut OutgoingStreamState,
}

impl<'a> OutgoingStream<'a> {
    pub fn push_framed(&mut self, bytes: Bytes) {
        self.push_frame_prefix(bytes.len());
        self.push_unframed(bytes);
    }

    pub fn push_chunks_framed<I: Iterator<Item = Bytes> + Clone>(&mut self, iter: I) {
        let size: usize = iter.clone().map(|v| v.len()).sum();
        self.push_frame_prefix(size);
        for bytes in iter { self.push_unframed(bytes); }
    }

    pub fn push_frame_prefix(&mut self, length: usize) {
        let varint = VarInt::try_from(length).unwrap();
        let mut buf = BytesMut::with_capacity(varint.len() as usize);
        varint.write(&mut buf).unwrap();
    }

    #[inline]
    pub fn push_unframed(&mut self, bytes: Bytes) {
        self.state.push(bytes);
    }
}

struct OutgoingStreamState {
    queue: WriteQueue,
}

impl OutgoingStreamState {
    fn push(&mut self, bytes: Bytes) {
        self.queue.push(bytes);
    }
}

struct WriteQueue(VecDeque<Bytes>);

impl WriteQueue {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    #[inline]
    pub fn push(&mut self, bytes: Bytes) {
        self.0.push_back(bytes);
    }

    pub fn write<S: StreamTryWrite>(&mut self, stream: &mut S) -> Option<StreamTryWriteOutcome> {
        if self.0.len() == 0 {
            return None;
        }

        while let Some(chunk) = self.0.pop_front() {
            match stream.try_write_stream(chunk.clone()) {
                StreamTryWriteOutcome::Complete => { continue },

                StreamTryWriteOutcome::Partial(written) => {
                    self.0.push_front(chunk.slice(written..));
                    return Some(StreamTryWriteOutcome::Partial(written));
                },

                StreamTryWriteOutcome::Blocked => {
                    self.0.push_front(chunk);
                    return Some(StreamTryWriteOutcome::Blocked);
                }

                StreamTryWriteOutcome::Error(err) => return Some(StreamTryWriteOutcome::Error(err)),
            }
        }

        return Some(StreamTryWriteOutcome::Complete);
    }
}