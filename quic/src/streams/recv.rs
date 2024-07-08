use std::collections::VecDeque;
use bevy_stardust_extras::numbers::VarInt;
use bytes::{Buf, Bytes};
use commitbuf::CommitBuf;
use super::*;

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

    pub fn ready(&self) -> bool {
        self.header.is_some()
    }

    pub fn header(&self) -> Option<StreamHeader> {
        self.header.clone()
    }

    pub fn iter<'a>(&'a mut self, limit: usize) -> Option<RecvIter> {
        if self.header.is_none() {
            if self.queue.len() == 0 { return None }
            let mut read = CommitBuf::new(&mut self.queue);

            self.header = Some(match StreamHeader::decode(&mut read) {
                Ok(header) => header,
                Err(_) => return None,
            });

            read.commit();
        }

        return Some(RecvIter {
            queue: &mut self.queue,
            limit,
        });
    }
}

pub(crate) struct RecvIter<'a> {
    queue: &'a mut VecDeque<Bytes>,
    limit: usize,
}

impl<'a> Iterator for RecvIter<'a> {
    type Item = Result<Bytes, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        // Try to read the length prefix
        let mut read = CommitBuf::new(&mut self.queue);
        let len: usize = match VarInt::read(&mut read) {
            Ok(len) => match u64::from(len).try_into() {
                Ok(len) => len,
                Err(_) => todo!(),
            },
            Err(_) => todo!(),
        };

        // Check that the length isn't above the limit
        if len > self.limit { return Some(Err(())); }

        // Check if enough data remains, if so we return it
        if read.remaining() < len { return None }
        let p = read.copy_to_bytes(len);
        read.commit();
        return Some(Ok(p))
    }
}