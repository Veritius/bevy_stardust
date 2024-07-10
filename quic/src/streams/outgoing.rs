use std::collections::VecDeque;
use bevy::utils::HashMap;
use bevy_stardust_extras::numbers::VarInt;
use bytes::{Bytes, BytesMut};
use super::{header::StreamPurpose, SendStream, StreamId, StreamTryWrite, StreamTryWriteOutcome};

pub(crate) struct OutgoingStreams {
    streams: HashMap<StreamId, OutgoingStreamState>,
}

impl OutgoingStreams {
    pub fn new() -> Self {
        Self {
            streams: HashMap::new(),
        }
    }

    pub fn open(&mut self, id: StreamId, purpose: StreamPurpose, transient: bool) {
        self.streams.insert(id, OutgoingStreamState {
            persistent: false,
            queue: WriteQueue::new(),
        });
    }

    #[must_use]
    pub fn get(&mut self, id: StreamId) -> Option<OutgoingStream> {
        let state = self.streams.get_mut(&id)?;
        return Some(OutgoingStream { state });
    }

    #[must_use]
    pub fn open_and_get(&mut self, id: StreamId, purpose: StreamPurpose, transient: bool) -> OutgoingStream {
        self.open(id, purpose, transient);
        return self.get(id).unwrap();
    }

    pub fn forget(&mut self, id: StreamId) {
        self.streams.remove(&id);
    }

    pub fn write<S: SendStream>(&mut self, id: StreamId, stream: &mut S) -> Option<OutgoingStreamsTryWriteOutcome> {
        let outgoing = self.streams.get_mut(&id)?;

        match outgoing.queue.write(stream)? {
            // Additional checks must be made if this is done
            StreamTryWriteOutcome::Complete => {
                if !outgoing.persistent {
                    stream.finish_stream();
                    self.forget(id);

                    // Send this event to inform that the stream was forgotten
                    return Some(OutgoingStreamsTryWriteOutcome::Finished(id));
                }

                // Return that we have completed the task
                return Some(OutgoingStreamsTryWriteOutcome::WriteOutcome(StreamTryWriteOutcome::Complete));
            },

            // Any other cases we forward with no further changes
            other => return Some(OutgoingStreamsTryWriteOutcome::WriteOutcome(other)),
        }
    }
}

pub(crate) struct OutgoingStream<'a> {
    state: &'a mut OutgoingStreamState,
}

impl<'a> OutgoingStream<'a> {
    pub fn make_persistent(&mut self) {
        self.state.persistent = true;
    }

    pub fn push_framed(&mut self, bytes: Bytes) {
        self.push_frame_prefix(bytes.len());
        self.push_unframed(bytes);
    }

    pub fn push_chunks_framed<I: Iterator<Item = Bytes>>(&mut self, iter: I) {
        // Push a dummy empty bytes we swap out later
        self.push_unframed(Bytes::new());
        let idx = self.state.queue.0.len();
        let mut length = 0;

        // Push all the bytes to the queue
        for bytes in iter {
            length += bytes.len();
            self.push_unframed(bytes);
        }

        // Replace the dummy bytes with the actual frame prefix
        let mut chunk = encode_varint(VarInt::try_from(length).unwrap());
        std::mem::swap(&mut self.state.queue.0[idx], &mut chunk);
        debug_assert_eq!(chunk.len(), 0);
    }

    pub fn push_frame_prefix(&mut self, length: usize) {
        let chunk = encode_varint(VarInt::try_from(length).unwrap());
        self.push_unframed(chunk);
    }

    #[inline]
    pub fn push_unframed(&mut self, bytes: Bytes) {
        if bytes.len() == 0 { return }
        self.state.push(bytes);
    }
}

pub(crate) enum OutgoingStreamsTryWriteOutcome {
    WriteOutcome(StreamTryWriteOutcome),
    Finished(StreamId),
}

struct OutgoingStreamState {
    persistent: bool,
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

fn encode_varint(varint: VarInt) -> Bytes {
    let mut buf = BytesMut::with_capacity(varint.len() as usize);
    varint.write(&mut buf).unwrap();
    return buf.freeze();
}