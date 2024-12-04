use std::{cmp::Ordering, collections::VecDeque};
use bytes::{Bytes, Buf};

/// A queue of individual [`Bytes`] that can be used as a [`Buf`], without copying the data.
/// 
/// Useful for data that is received piecemeal and must be deserialised as a whole.
#[derive(Default)]
pub struct ChunkStream {
    queue: VecDeque<Bytes>,
}

impl ChunkStream {
    /// Creates a new, empty [`ChunkStream`].
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// Pushes a chunk to the back of the queue.
    #[inline]
    pub fn push(&mut self, chunk: Bytes) {
        self.queue.push_back(chunk);
    }

    /// Clears the queue but does not reallocate.
    #[inline]
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

impl Buf for ChunkStream {
    fn remaining(&self) -> usize {
        self.queue.iter().map(|b| b.len()).sum()
    }

    fn chunk(&self) -> &[u8] {
        &self.queue[0]
    }

    fn advance(&mut self, mut cnt: usize) {
        while cnt > 0 {
            let front = &mut self.queue[0];

            match front.len().cmp(&cnt) {
                Ordering::Less | Ordering::Equal => {
                    cnt -= front.len();
                    self.queue.pop_front();
                },

                Ordering::Greater => {
                    *front = front.slice(cnt..);
                    cnt = 0;
                },
            }
        }
    }
}

#[test]
fn stream_chunk_buf_test() {
    let mut buf = ChunkStream::new();

    buf.push(Bytes::from_static(b"Hello,")); // 6
    buf.push(Bytes::from_static(b"")); // 0
    buf.push(Bytes::from_static(b" ")); // 1
    buf.push(Bytes::from_static(b"world!")); // 6

    assert_eq!(buf.remaining(), 13);

    buf.advance(3);

    assert_eq!(buf.remaining(), 10);

    buf.advance(5);

    assert_eq!(buf.remaining(), 5);

    buf.advance(3);

    assert_eq!(buf.remaining(), 2);
}