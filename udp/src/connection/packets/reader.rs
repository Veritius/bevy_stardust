use std::collections::VecDeque;
use bytes::Bytes;
use crate::connection::reliability::ReliablePackets;
use super::frames::Frame;

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
    pub fn iter<'a>(&'a mut self, context: PacketReaderContext<'a>) -> PacketReaderIter<'a> {
        todo!()
    }

    pub(in crate::connection) fn push(&mut self, packet: Bytes) {
        self.queue.push_back(packet)
    }
}

pub(crate) struct PacketReaderContext<'a> {
    pub reliability: &'a mut ReliablePackets,
}

pub(crate) struct PacketReaderIter<'a> {
    inner: &'a mut PacketReader,
    ctx: PacketReaderContext<'a>,
}

impl Iterator for PacketReaderIter<'_> {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}