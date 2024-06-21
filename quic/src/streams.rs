use std::cmp::Ordering;

use bevy::utils::{smallvec::{smallvec, SmallVec}, thiserror::Error};
use bevy_stardust::prelude::*;
use quinn_proto::{coding::{Codec, Result as DecodeResult, UnexpectedEnd}, SendStream, VarInt, WriteError};

pub(crate) enum StreamOpenHeader {
    StardustReliable {
        channel: ChannelId,
    },
}

impl StreamOpenHeader {
    const STARDUST_RELIABLE: VarInt = VarInt::from_u32(0);
}

impl Codec for StreamOpenHeader {
    fn encode<B: BufMut>(&self, buf: &mut B) {
        match self {
            StreamOpenHeader::StardustReliable { channel } => {
                Self::STARDUST_RELIABLE.encode(buf);
                VarInt::from_u32(channel.clone().into()).encode(buf)
            },
        }
    }

    fn decode<B: Buf>(buf: &mut B) -> DecodeResult<Self> {
        match VarInt::decode(buf)? {
            Self::STARDUST_RELIABLE => Ok(Self::StardustReliable {
                channel: {
                    let vint = VarInt::decode(buf)?.into_inner();
                    // TODO: Output a good error type instead of unexpected end
                    let uint: u32 = vint.try_into().map_err(|_| UnexpectedEnd)?;
                    ChannelId::from(uint)
                },
            }),

            _ => Err(UnexpectedEnd),
        }
    }
}

struct FramedMessageHeader {
    length: VarInt,
}

impl Codec for FramedMessageHeader {
    fn encode<B: BufMut>(&self, buf: &mut B) {
        self.length.encode(buf);
    }

    fn decode<B: Buf>(buf: &mut B) -> DecodeResult<Self> {
        Ok(Self {
            length: VarInt::decode(buf)?,
        })
    }
}

pub(crate) struct FramedWriter {
    buffer: SmallVec<[Bytes; 2]>,
}

impl FramedWriter {
    pub fn new() -> Self {
        Self {
            buffer: SmallVec::new(),
        }
    }

    pub fn queue(&mut self, bytes: Bytes) {
        // Create the header data
        let mut buf = BytesMut::new();
        FramedMessageHeader {
            length: VarInt::from_u64(bytes.len() as u64).unwrap(),
        }.encode(&mut buf);

        // Push segments to the buffer
        self.buffer.push(buf.freeze());
        self.buffer.push(bytes);
    }

    pub fn write(&mut self, stream: &mut SendStream) -> Result<usize, WriteError> {
        // Some working space stuff
        let mut written = 0;
        let mut swap: SmallVec<[Bytes; 2]> = SmallVec::with_capacity(self.buffer.len());
        let mut drain = self.buffer.drain(..);

        // Write as many chunks as possible
        while let Some(bytes) = drain.next() {
            let len = bytes.len();
            match stream.write(&bytes[..]) {
                // Partial write
                Ok(amt) if amt != len => {
                    written += amt;
                    swap.push(bytes.slice(amt..));
                }

                // Full write
                Ok(amt) => {
                    written += amt;
                },

                // Stream is blocked
                Err(err) if err == WriteError::Blocked => {
                    swap.push(bytes);
                    break;
                }

                // Error during write
                Err(err) => {
                    return Err(err);
                },
            }
        }

        // Finish up
        swap.extend(drain);
        self.buffer = swap;
        return Ok(written);
    }

    #[inline]
    pub fn unsent(&self) -> usize {
        self.buffer.iter().map(|v| v.len()).sum()
    }
}

pub(crate) struct StreamReader {
    queue: SmallVec<[Bytes; 1]>,
    state: StreamReaderState,
}

impl Default for StreamReader {
    fn default() -> Self {
        Self {
            queue: SmallVec::new(),
            state: StreamReaderState::ParseHeader,
        }
    }
}

impl StreamReader {
    pub fn push(&mut self, bytes: Bytes) {
        self.queue.push(bytes);
    }

    pub fn next<'a>(&'a mut self) -> Result<Option<StreamReaderSegment>, StreamReadError> {
        if self.queue.len() == 0 { return Ok(None); }

        match self.state {
            StreamReaderState::ParseHeader => todo!(),
            StreamReaderState::Stardust { channel } => todo!(),
        }

        todo!()
    }

    #[inline]
    pub fn unread(&self) -> usize {
        self.queue.iter().map(|v| v.len()).sum()
    }
}

enum StreamReaderState {
    ParseHeader,

    Stardust {
        channel: ChannelId,
    },
}

pub(crate) enum StreamReaderSegment {
    Stardust {
        channel: ChannelId,
        payload: Bytes,
    },
}

#[derive(Debug, Clone, Copy, Error)]
pub(crate) enum StreamReadError {
    #[error("length exceeded maximum")]
    LengthExceededMaximum,
}

struct Burner<'a> {
    consumed: usize,
    remaining: usize,
    cursor: usize,
    index: usize,
    inner: &'a mut SmallVec<[Bytes; 1]>,
}

impl<'a> Burner<'a> {
    fn new(vec: &'a mut SmallVec<[Bytes; 1]>) -> Self {
        Self {
            consumed: 0,
            remaining: vec.iter().map(|v| v.len()).sum(),
            cursor: 0,
            index: 0,
            inner: vec,
        }
    }

    fn commit(&mut self) {
        self.consumed = 0;

        if self.cursor > 0 {
            let m = &mut self.inner[self.cursor];
            *m = m.slice(self.cursor..);
            self.cursor = 0;
        }

        if self.index == 0 { return }

        let mut swap: SmallVec<[Bytes; 1]> = SmallVec::with_capacity(self.inner.len() - self.index);
        while self.index > 0 {
            swap.push(self.inner.pop().unwrap());
            self.index -= 1;
        }

        core::mem::swap(self.inner, &mut swap);
        drop(swap);
    }

    fn reset(&mut self) {
        self.consumed = 0;
        self.cursor = 0;
        self.index = 0;
        self.remaining = self.inner.iter().map(|v| v.len()).sum();
    }
}

impl Buf for Burner<'_> {
    #[inline]
    fn remaining(&self) -> usize {
        self.remaining
    }

    fn chunk(&self) -> &[u8] {
        let sel = &self.inner[self.index];
        return &sel[self.cursor..];
    }

    fn advance(&mut self, cnt: usize) {
        if cnt > self.remaining { panic!() }
        self.remaining -= cnt;
        self.consumed += cnt;

        let sel = &self.inner[self.index];
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

impl Drop for Burner<'_> {
    fn drop(&mut self) {
        debug_assert!(self.consumed == 0, "Burner was dropped but not committed");
    }
}

#[test]
fn burner_test() {
    let mut slices: SmallVec<[Bytes; 1]> = smallvec![
        Bytes::from_static(b"Hello"),
        Bytes::from_static(b", "),
        Bytes::from_static(b"wor"),
        Bytes::from_static(b"ld!"),
    ];

    let mut burner = Burner::new(&mut slices);
    assert_eq!(burner.remaining(), 13);
    assert_eq!(burner.chunk(), b"Hello");

    assert_eq!(burner.get_u8(), b'H');
    assert_eq!(burner.get_u8(), b'e');
    assert_eq!(burner.chunk(), b"llo");
    burner.advance(3);
    assert_eq!(burner.remaining(), 8);

    assert_eq!(burner.chunk(), b", ");
    burner.advance(2);
    assert_eq!(burner.remaining(), 6);

    assert_eq!(burner.chunk(), b"wor");
    burner.advance(3);
    assert_eq!(burner.remaining(), 3);

    assert_eq!(burner.chunk(), b"ld!");
    burner.advance(3);
    assert_eq!(burner.remaining(), 0);

    burner.commit();
}