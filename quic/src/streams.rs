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