use std::collections::VecDeque;
use bevy::utils::HashMap;
use bevy_stardust::messages::ChannelId;
use bevy_stardust_extras::numbers::VarInt;
use bytes::{Bytes, BytesMut};
use super::{StreamId, StreamTryWrite, StreamTryWriteOutcome};

pub(crate) struct OutgoingStreams {
    channels: HashMap<ChannelId, StreamId>,
    streams: HashMap<StreamId, OutgoingStream>,
}

impl OutgoingStreams {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            streams: HashMap::new(),
        }
    }

    pub fn write<S: StreamTryWrite>(&mut self, id: StreamId, stream: &mut S) -> Option<StreamTryWriteOutcome> {
        let outgoing = self.streams.get_mut(&id)?;
        return outgoing.queue.write(stream);
    }
}

struct OutgoingStream {
    framed: bool,
    queue: WriteQueue,
}

impl OutgoingStream {
    fn push(&mut self, bytes: Bytes) {
        if self.framed {
            let varint = VarInt::try_from(bytes.len()).unwrap();
            let mut buf = BytesMut::with_capacity(varint.len() as usize);
            varint.write(&mut buf).unwrap();
            self.queue.push(buf.freeze());
        }

        self.queue.push(bytes);
    }
}

struct WriteQueue(VecDeque<Bytes>);

impl WriteQueue {
    #[inline]
    pub fn push(&mut self, bytes: Bytes) {
        self.0.push_back(bytes);
    }

    pub fn write<S: StreamTryWrite>(&mut self, stream: &mut S) -> Option<StreamTryWriteOutcome> {
        if self.0.len() == 0 {
            return None;
        }

        while let Some(chunk) = self.0.pop_front() {
            match stream.try_write(chunk.clone()) {
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