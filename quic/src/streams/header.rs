use bytes::{Buf, BufMut};
use quinn_proto::{VarInt, coding::Codec};

pub(crate) enum StreamHeader {
    Stardust {
        channel: u32,
    }
}

impl StreamHeader {
    pub fn encode<B: BufMut>(&self, buffer: &mut B) {
        match self {
            StreamHeader::Stardust { channel } => {
                VarInt::from_u32(0).encode(buffer);
                VarInt::from_u32(*channel).encode(buffer);
            },
        }
    }

    pub fn decode<B: Buf>(buffer: &mut B) -> Result<Self, ()> {
        let code = VarInt::decode(buffer).map_err(|_| ())?;

        match code.into_inner() {
            0 => Ok(Self::Stardust {
                channel: VarInt::decode(buffer).map_err(|_| ())?.into_inner() as u32,
            }),

            _ => Err(todo!()),
        }
    }
}