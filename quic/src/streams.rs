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
    buffer: SmallVec<[Bytes; 1]>,
}

impl FramedWriter {
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
        // Create the header data
        let mut buf = BytesMut::new();
        FramedMessageHeader {
            length: VarInt::from_u64(bytes.len() as u64).unwrap(),
        }.encode(&mut buf);

        // Push segments to the buffer
        self.buffer.push(buf.freeze());
        self.buffer.push(bytes);
    }

    pub fn pop(&mut self, space: usize) -> Option<Bytes> {
        // TODO: Optimise this, unnecessarily shifts occur
        match self.buffer.len() {
            0 => None,
            1 => self.buffer.pop(),
            _ => Some(self.buffer.remove(0)),
        }
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

    pub fn pop(&mut self) -> Option<Bytes> {
        todo!()
    }
}

/// A framed message that can be sent over a stream.
#[derive(Debug, Clone)]
pub(crate) struct FramedMessage {
    pub payload: Bytes,
}

impl FramedMessage {
    pub fn write(self, buf: &mut Vec<u8>, stream: &mut SendStream) -> Result<usize, WriteError> {
        // Counter for written bytes
        let mut written = 0;

        // Write the length of the message
        VarInt::try_from(self.payload.len()).unwrap().encode(buf);
        written += stream.write(buf)?;
        buf.clear();

        // Write the payload itself
        written += stream.write(&self.payload[..])?;

        // Return amount of bytes that are written
        return Ok(written)
    }
}