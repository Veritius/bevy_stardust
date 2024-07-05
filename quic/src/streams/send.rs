use std::collections::VecDeque;
use bytes::{Bytes, BytesMut};
use header::StreamHeader;
use quinn_proto::{VarInt, coding::Codec};
use super::*;

pub(crate) struct Send {
    framed: bool,
    transient: bool,
    queue: VecDeque<Bytes>,
}

impl Send {
    pub fn new(init: SendInit) -> Self {
        let mut queue = VecDeque::with_capacity(1);

        let framed = match init {
            SendInit::StardustPersistent { channel: _ } => true,
            SendInit::StardustTransient  { channel: _ } => true,
        };

        let transient = match init {
            SendInit::StardustPersistent { channel: _ } => false,
            SendInit::StardustTransient  { channel: _ } => true,
        };

        let header = match init {
            SendInit::StardustPersistent { channel } |
            SendInit::StardustTransient  { channel } => StreamHeader::Stardust { channel },
        };

        let mut buffer = BytesMut::with_capacity(8);
        header.encode(&mut buffer);
        queue.push_back(buffer.freeze());

        return Self { framed, transient, queue };
    }

    pub fn push(&mut self, chunk: Bytes) {
        if self.framed {
            let mut buffer = BytesMut::with_capacity(4);
            VarInt::from_u64(chunk.len() as u64).unwrap().encode(&mut buffer);
            self.queue.push_back(buffer.freeze());
        }

        self.queue.push_back(chunk);
    }

    pub fn transient(&self) -> bool {
        self.transient
    }
}

impl StreamWriter for Send {
    fn write<S: WritableStream>(&mut self, stream: &mut S) -> StreamWriteOutcome {
        let mut total = 0;
        let mut written = 0;

        while let Some(bytes) = self.queue.pop_front() {
            total += bytes.len();

            match stream.write_to(bytes.clone()) {
                // A complete write means we can try again
                StreamWriteOutcome::Complete => {
                    written += bytes.len();
                    continue;
                },

                // A partial write means we have to stop
                StreamWriteOutcome::Partial(amt) => {
                    written += amt;
                    let bytes = bytes.slice(amt..);
                    self.queue.push_front(bytes);
                    break;
                },

                // A block error means we must stop writing
                StreamWriteOutcome::Blocked => {
                    self.queue.push_front(bytes);
                    break;
                }

                // An error means the stream can no longer be written to
                StreamWriteOutcome::Error(err) => {
                    self.queue.push_front(bytes);
                    return StreamWriteOutcome::Error(err);
                },
            }
        }

        return match total == written {
            true => StreamWriteOutcome::Complete,
            false => StreamWriteOutcome::Partial(written),
        };
    }
}

pub(crate) enum SendInit {
    StardustPersistent { channel: u32 },
    StardustTransient  { channel: u32 },
}