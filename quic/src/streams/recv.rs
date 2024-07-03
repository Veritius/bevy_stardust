use std::collections::VecDeque;
use bytes::Bytes;
use commitbuf::CommitBuf;
use super::*;
use crate::*;

pub(crate) struct Recv {
    state: RecvState,
    queue: VecDeque<Bytes>,
}

impl StreamReader for Recv {
    fn read<S: ReadableStream>(&mut self, stream: &mut S) -> Result<usize, StreamReadError> {
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

    pub fn poll(&mut self, config: &QuicConfig) -> RecvOutput {
        if self.state.is_nothing() {
            if self.queue.len() == 0 { return RecvOutput::Nothing }
            let mut read = CommitBuf::new(&mut self.queue);

            let header = match StreamHeader::decode(&mut read) {
                Ok(header) => header,
                Err(_) => return RecvOutput::Nothing,
            };

            read.commit();

            self.state = match header {
                StreamHeader::Stardust { channel } => RecvState::Stardust { channel },
            }
        }

        match self.state {
            RecvState::Unknown => unreachable!(),

            RecvState::Stardust { channel } => RecvOutput::Stardust(StardustRecv {
                queue: &mut self.queue,
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
        todo!()
    }
}