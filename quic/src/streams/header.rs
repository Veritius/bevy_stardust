use bevy_stardust::prelude::*;
use bevy_stardust_extras::numbers::VarInt;
use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug, Clone, Copy)]
pub(super) enum StreamHeader {
    Stardust {
        channel: ChannelId,
    },

    WrappedDatagram,
}

impl StreamHeader {
    pub fn read<B: Buf>(buf: &mut B) -> Result<StreamHeader, ()> {
        let code: u64 = VarInt::read(buf)?.into();

        match code {
            0 => {
                let channel = VarInt::read(buf)
                    .and_then(|v| u32::try_from(v))
                    .map(|v| ChannelId::from(v))?;

                return Ok(StreamHeader::Stardust {
                    channel,
                });
            },

            1 => return Ok(StreamHeader::WrappedDatagram),

            _ => return Err(()),
        }
    }

    pub fn write<B: BufMut>(&self, buf: &mut B) -> Result<(), ()> {
        match self {
            StreamHeader::Stardust { channel } => {
                VarInt::from_u32(0).write(buf)?;
                VarInt::from_u32((*channel).into()).write(buf)?;
            },

            StreamHeader::WrappedDatagram => {
                VarInt::from_u32(1).write(buf)?;
            }
        }

        return Ok(());
    }

    pub fn alloc(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(8);
        self.write(&mut buf);
        return buf.freeze();
    }
}