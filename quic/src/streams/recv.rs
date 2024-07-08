use std::collections::VecDeque;
use bytes::Bytes;
use commitbuf::CommitBuf;
use super::*;
use crate::*;

pub(crate) struct Recv {
    header: Option<StreamHeader>,
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
            header: None,
            queue: VecDeque::with_capacity(1),
        }
    }

    pub fn header(&self) -> Option<StreamHeader> {
        self.header.clone()
    }

    pub fn poll<'a>(&'a mut self, config: &'a QuicConfig) -> Option<RecvIter> {
        if self.header.is_none() {
            if self.queue.len() == 0 { return None }
            let mut read = CommitBuf::new(&mut self.queue);

            self.header = Some(match StreamHeader::decode(&mut read) {
                Ok(header) => header,
                Err(_) => return None,
            });

            read.commit();
        }

        return Some(RecvIter { queue: &mut self.queue });
    }
}

pub(crate) struct RecvIter<'a> {
    queue: &'a mut VecDeque<Bytes>,
}