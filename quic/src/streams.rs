use bevy_stardust::prelude::*;
use quinn_proto::{VarInt, coding::{Codec, UnexpectedEnd, Result as DecodeResult}};

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

pub(crate) struct StreamFrameHeader {
    pub length: usize,
}

impl Codec for StreamFrameHeader {
    fn encode<B: BufMut>(&self, buf: &mut B) {
        // Encode the length as a variable length integer
        // This is fine since I don't think anyone will
        // attempt to send a message bigger than 4,000 petabytes
        VarInt::try_from(self.length).unwrap().encode(buf);
    }

    fn decode<B: Buf>(buf: &mut B) -> DecodeResult<Self> {
        Ok(Self {
            length: u64::from(VarInt::decode(buf)?) as usize,
        })
    }
}