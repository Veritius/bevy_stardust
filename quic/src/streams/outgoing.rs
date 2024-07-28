use std::collections::VecDeque;
use bytes::{Bytes, BytesMut};
use super::framing::FramedHeader;

pub(crate) struct OutgoingStream {
    queue: VecDeque<Bytes>,
}

impl OutgoingStream {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub(super) fn push_unframed(&mut self, bytes: Bytes) {
        self.queue.push_back(bytes)
    }

    pub(super) fn push_framed(&mut self, bytes: Bytes) {
        let framing = FramedHeader {
            length: bytes.len(),
        };

        let mut buf = BytesMut::with_capacity(8);
        framing.write(&mut buf).expect("BytesMut buffer was too small");
        self.push_unframed(buf.freeze());

        self.push_unframed(bytes)
    }

    pub(super) fn push_chunks_unframed<I>(&mut self, iter: I) -> usize
    where
        I: IntoIterator<Item = Bytes>,
    {
        let mut length = 0;

        for bytes in iter {
            length += bytes.len();
            self.push_unframed(bytes);
        }

        return length;
    }

    pub(super) fn push_chunks_framed<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Bytes>,
    {
        // Push a dummy empty bytes we swap out later
        self.push_unframed(Bytes::new());
        let idx = self.queue.len();

        // Push all the bytes to the queue
        let length = self.push_chunks_unframed(iter);

        // Create the framing header and serialise it
        let framing = FramedHeader { length };
        let mut buf = BytesMut::with_capacity(8);
        framing.write(&mut buf).expect("BytesMut buffer was too small");
        let mut chunk = buf.freeze();

        // Replace the dummy bytes with the actual frame prefix
        std::mem::swap(&mut self.queue[idx], &mut chunk);
        debug_assert_eq!(chunk.len(), 0);
    }
}