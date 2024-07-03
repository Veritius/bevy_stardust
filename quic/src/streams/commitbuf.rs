use std::{cmp::Ordering, collections::VecDeque};
use bytes::{Buf, Bytes};

pub(super) struct CommitBuf<'a> {
    remaining: usize,
    cursor: usize,
    index: usize,
    queue: &'a mut VecDeque<Bytes>,
}

impl<'a> CommitBuf<'a> {
    pub fn new(queue: &'a mut VecDeque<Bytes>) -> CommitBuf<'a> {
        Self {
            remaining: queue.iter().map(|v| v.len()).sum(),
            cursor: 0,
            index: 0,
            queue,
        }
    }

    pub fn commit(self) {
        let mut consumed = self.queue.iter().map(|v| v.len()).sum::<usize>() - self.remaining;

        while consumed > 0 {
            let f = self.queue.pop_front().unwrap();

            if consumed >= f.len() {
                consumed -= f.len(); 
                continue;
            }

            self.queue.push_front(f.slice(consumed..));
            consumed -= f.len();
        }
    }
}

impl<'a> Buf for CommitBuf<'a> {
    #[inline]
    fn remaining(&self) -> usize {
        self.remaining
    }

    #[inline]
    fn chunk(&self) -> &[u8] {
        &self.queue[self.index][self.cursor..]
    }

    fn advance(&mut self, cnt: usize) {
        if cnt > self.remaining { panic!("Overran buffer"); }
        self.remaining -= cnt;

        let sel = &self.queue[self.index];
        match (self.cursor + cnt).cmp(&sel.len()) {
            Ordering::Less => {
                self.cursor += cnt;
            },

            Ordering::Equal => {
                self.cursor = 0;
                self.index += 1;
            },

            Ordering::Greater => {
                self.cursor = cnt - sel.len();
                self.index += 1;
            },
        }
    }
}