use std::collections::VecDeque;
use bevy_stardust::channels::id::ChannelId;
use bytes::Bytes;
use quinn_proto::{SendStream, StreamId, WriteError};

pub(crate) struct OutgoingStreamData {
    pub id: StreamId,
    buffer: Box<[u8]>,
}

impl OutgoingStreamData {
    pub fn new(id: StreamId, prefix: &[u8]) -> Self {
        Self {
            id,
            buffer: Box::from(prefix),
        }
    }

    pub fn push(&mut self, data: &[u8]) {
        if data.len() == 0 { return }

        dbg!(self.buffer.len(), data.len());

        let mut buf: Vec<u8> = Vec::with_capacity(self.buffer.len() + data.len());
        buf.extend_from_slice(&self.buffer);
        buf.extend_from_slice(data);

        self.buffer = buf.into();
    }

    pub fn try_write(&mut self, stream: &mut SendStream) -> Result<usize, WriteError> {
        match stream.write(&self.buffer) {
            Ok(bytes) => {
                if bytes == 0 { return Ok(bytes) }
                let mut buf = Vec::with_capacity(self.buffer.len() - bytes);
                buf.extend(&self.buffer[bytes..]);
                self.buffer = buf.into();
                Ok(bytes)
            },
            Err(err) => Err(err),
        }
    }

    pub fn is_drained(&self) -> bool {
        self.buffer.len() == 0
    }
}

pub(crate) struct IncomingStreamData {
    pub id: ChannelId,
    buffer: Box<[u8]>,
    plds: VecDeque<Bytes>,
}

impl IncomingStreamData {
    pub fn new(id: ChannelId) -> Self {
        Self {
            id,
            buffer: Box::new([]),
            plds: VecDeque::new(),
        }
    }

    pub fn next(&mut self) -> Option<Bytes> {
        self.plds.pop_front()
    }

    pub fn read(&mut self, bytes: &[u8]) {
        todo!()
    }
}