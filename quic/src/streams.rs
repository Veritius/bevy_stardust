use std::collections::VecDeque;
use bevy_stardust::channels::{id::ChannelId, registry::ChannelRegistry};
use bytes::Bytes;
use quinn_proto::{SendStream, StreamId, WriteError};

#[repr(u8)]
pub(crate) enum StreamPurposeHeader {
    ConnectionEvents = 0,
    StardustPayloads = 1,
}

impl TryFrom<u8> for StreamPurposeHeader {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::ConnectionEvents,
            1 => Self::StardustPayloads,
            _ => return Err(())
        })
    }
}

pub(crate) struct OutgoingBufferedStreamData {
    pub id: StreamId,
    buffer: Box<[u8]>,
}

impl OutgoingBufferedStreamData {
    pub fn new(id: StreamId) -> Self {
        Self {
            id,
            buffer: Box::default(),
        }
    }

    pub fn push(&mut self, data: &[u8]) {
        if data.len() == 0 { return }

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

pub(crate) struct IncomingStardustStreamData {
    pub id: ChannelId,
    buffer: Box<[u8]>,
    plds: VecDeque<Bytes>,
}

impl IncomingStardustStreamData {
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

    pub fn read(&mut self, registry: &ChannelRegistry, bytes: &[u8]) {
        todo!()
    }
}