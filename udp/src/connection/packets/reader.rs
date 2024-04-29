use std::collections::VecDeque;
use bytes::Bytes;

pub(in crate::connection) struct PacketReader {

}

impl PacketReader {
    pub fn iter<'a>(&'a mut self, queue: &'a mut VecDeque<Bytes>) -> PacketReaderIter<'a> {
        todo!()
    }
}

pub(in crate::connection) struct PacketReaderIter<'a> {
    inner: &'a mut PacketReader,
    queue: &'a mut VecDeque<Bytes>,
}