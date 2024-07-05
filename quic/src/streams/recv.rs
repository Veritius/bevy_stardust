use std::collections::VecDeque;
use bevy::log::trace;
use bytes::{Buf, Bytes};
use commitbuf::CommitBuf;
use quinn_proto::{VarInt, coding::Codec};
use super::*;
use crate::*;

pub(crate) struct Recv {
    state: RecvState,
    queue: VecDeque<Bytes>,
}

impl StreamReader for Recv {
    fn read_from<S: ReadableStream>(&mut self, stream: &mut S) -> Result<usize, StreamReadError> {
        let mut read = 0;

        loop {
            match stream.read() {
                StreamReadOutcome::Chunk(chunk) => {
                    read += chunk.len();
                    self.queue.push_back(chunk);
                },

                StreamReadOutcome::Blocked => { break },

                StreamReadOutcome::Finished => { break },

                StreamReadOutcome::Error(err) => return Err(err),
            }
        }

        return Ok(read)
    }
}

impl Recv {
    pub fn new() -> Self {
        Self {
            state: RecvState::Unknown,
            queue: VecDeque::with_capacity(1),
        }
    }

    pub fn poll<'a>(&'a mut self, config: &'a QuicConfig) -> RecvOutput<'a> {
        if self.state.is_nothing() {
            if self.queue.len() == 0 { return RecvOutput::Nothing }
            let mut read = CommitBuf::new(&mut self.queue);

            let header = match StreamHeader::decode(&mut read) {
                Ok(header) => header,
                Err(_) => return RecvOutput::Nothing,
            };

            read.commit();

            self.state = match header {
                StreamHeader::Stardust { channel } => {
                    trace!(channel, "Received stream for Stardust messages");
                    RecvState::Stardust { channel }
                },
            }
        }

        match self.state {
            RecvState::Unknown => unreachable!(),

            RecvState::Stardust { channel } => RecvOutput::Stardust(StardustRecv {
                queue: &mut self.queue,
                limit: config.maximum_framed_message_length,
                channel,
            })
        }
    }
}

enum RecvState {
    Unknown,

    Stardust {
        channel: u32,
    }
}

impl RecvState {
    fn is_nothing(&self) -> bool {
        match self {
            RecvState::Unknown => true,
            _ => false,
        }
    }
}

pub(crate) enum RecvOutput<'a> {
    Nothing,

    Stardust(StardustRecv<'a>),
}

pub(crate) struct StardustRecv<'a> {
    queue: &'a mut VecDeque<Bytes>,
    limit: usize,
    channel: u32,
}

impl StardustRecv<'_> {
    pub fn channel(&self) -> u32 {
        self.channel
    }
}

impl<'a> Iterator for StardustRecv<'a> {
    type Item = Bytes;

    fn next(&mut self) -> Option<Self::Item> {
        let mut read = CommitBuf::new(&mut self.queue);
        let length = VarInt::decode(&mut read).ok()?.into_inner() as usize;

        if length > self.limit {
            todo!()
        }

        if length > read.remaining() { return None; }

        let payload = read.copy_to_bytes(length);
        read.commit();

        return Some(payload);
    }
}