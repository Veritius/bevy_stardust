use std::collections::VecDeque;
use bevy_stardust::channels::id::ChannelId;
use bytes::Bytes;
use quinn_proto::{SendStream, StreamId, WriteError};

pub(crate) struct OutgoingStreamData {
    pub id: StreamId,
    buffer: Box<[u8]>,
}

impl OutgoingStreamData {
    pub fn push(&mut self, data: &[u8]) {
        let cur_len = self.buffer.len();
        let dat_len = data.len();
        let max_len = cur_len + dat_len;

        let mut buf: Vec<u8> = Vec::with_capacity(max_len);
        buf[..cur_len].copy_from_slice(&self.buffer);
        buf[cur_len..max_len].copy_from_slice(data);

        self.buffer = buf.into();
    }

    pub fn try_write(&mut self, stream: &mut SendStream) -> Result<usize, WriteError> {
        match stream.write(&self.buffer) {
            Ok(bytes) => {
                let mut buf = Vec::with_capacity(self.buffer.len() - bytes);
                buf[bytes..].copy_from_slice(&self.buffer[bytes..]);
                self.buffer = buf.into();
                Ok(bytes)
            },
            Err(err) => Err(err),
        }
    }
}

pub(crate) struct IncomingStreamData {
    pub id: ChannelId,
    buffer: Box<[u8]>,
    plds: VecDeque<Bytes>,
}

impl IncomingStreamData {
    pub fn next(&mut self) -> Option<Bytes> {
        self.plds.pop_front()
    }

    pub fn read(&mut self, bytes: &[u8]) {
        todo!()
    }
}