use std::collections::VecDeque;
use bytes::Bytes;
use tracing::error;
use unbytes::Reader;
use crate::connection::reliability::ReliablePackets;
use super::frames::RecvFrame;

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
        PacketReaderIter { inner: self, current: None, context }
    }

    pub(in crate::connection) fn push(
        &mut self,
        packet: Bytes,
    ) {
        self.queue.push_back(packet)
    }
}

pub(crate) struct PacketReaderContext<'a> {
    pub reliability: &'a mut ReliablePackets,
}

/// Dropping this type may cause data loss.
/// Use [`is_safe_to_drop`](Self::is_safe_to_drop) to check if you can drop this without data loss.
pub(crate) struct PacketReaderIter<'a> {
    inner: &'a mut PacketReader,
    current: Option<Reader>,
    context: PacketReaderContext<'a>,
}

impl PacketReaderIter<'_> {
    #[inline]
    pub fn is_safe_to_drop(&self) -> bool {
        self.current.is_none()
    }
}

impl Drop for PacketReaderIter<'_> {
    fn drop(&mut self) {
        if !self.is_safe_to_drop() {
            error!("PacketReaderIter was dropped with unread data");
        }
    }
}

impl Iterator for PacketReaderIter<'_> {
    type Item = Result<RecvFrame, PacketReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_none() {
            let bytes = self.inner.queue.pop_front()?;
            self.current = Some(Reader::new(bytes));
        }

        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PacketReadError {

}