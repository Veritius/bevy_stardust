use std::collections::VecDeque;
use bytes::Bytes;

/// Parses incoming packets into an iterator of `Frame` objects.
pub(crate) struct PacketReader {
    queue: VecDeque<Bytes>,
}

impl Default for PacketReader {
    fn default() -> Self {
        Self {
            queue: VecDeque::with_capacity(16),
        }
    }
}

impl PacketReader {
    pub fn iter<'a>(&'a mut self) -> PacketReaderIter<'a> {
        todo!()
    }

    pub(in crate::connection) fn push(&mut self, packet: Bytes) {
        self.queue.push_back(packet)
    }
}

pub(crate) struct PacketReaderIter<'a> {
    inner: &'a mut PacketReader,
}