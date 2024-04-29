use bytes::Bytes;
use crate::connection::reliability::ReliablePackets;

use super::frames::Frame;

/// Packs a queue of `Frame` objects into a single packet.
pub(crate) struct PacketBuilder {
    queue: Vec<Frame>,
}

impl Default for PacketBuilder {
    fn default() -> Self {
        Self {
            queue: Vec::with_capacity(32),
        }
    }
}

impl PacketBuilder {
    pub fn iter<'a>(&'a mut self, context: PacketBuilderContext<'a>) -> PacketBuilderIter<'a> {
        // Sort the queue by priority using Frame's Ord impl
        self.queue.sort_unstable();

        todo!()
    }

    pub(in crate::connection) fn push(&mut self, frame: Frame) {
        self.queue.push(frame);
    }
}

pub(crate) struct PacketBuilderContext<'a> {
    pub reliability: &'a mut ReliablePackets,
}

pub(crate) struct PacketBuilderIter<'a> {
    inner: &'a mut PacketBuilder,
    ctx: PacketBuilderContext<'a>,
}

impl Iterator for PacketBuilderIter<'_> {
    type Item = Bytes;
    
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}