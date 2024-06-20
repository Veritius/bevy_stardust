use bevy::utils::smallvec::SmallVec;
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

    pub fn unsent(&self) -> usize {
        self.buffer.iter().map(|v| v.len()).sum()
    }
}

pub(crate) struct FramedReader {
    buffer: SmallVec<[Bytes; 1]>,
}

impl FramedReader {
    pub fn new() -> Self {
        Self {
            buffer: SmallVec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: SmallVec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, bytes: Bytes) {
        self.buffer.push(bytes);
    }

    pub fn unread(&self) -> usize {
        self.buffer.iter().map(|bytes| bytes.len()).sum()
    }

    pub fn read(&mut self) -> impl Iterator<Item = Bytes> {
        struct Dn {}
        impl Iterator for Dn {
            type Item = Bytes;
            fn next(&mut self) -> Option<Self::Item> {
                unimplemented!()
            }
        }

        todo!();
        return Dn {};
    }
}