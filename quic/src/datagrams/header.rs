use bevy_stardust::prelude::*;
use bevy_stardust_extras::numbers::VarInt;
use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy)]
pub(super) struct DatagramHeader {
    pub purpose: DatagramPurpose,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum DatagramPurpose {
    Stardust { channel: ChannelId },
}

#[derive(Clone, Copy)]
#[repr(u32)]
enum DatagramPurposeCode {
    Stardust = 0,
}

impl DatagramHeader {
    pub fn encode_len(&self) -> usize {
        let mut len: usize = 0;

        len += VarInt::from_u32(self.purpose.code() as u32).len() as usize;

        match self.purpose {
            DatagramPurpose::Stardust { channel } => {
                len += VarInt::from_u32(self.purpose.code() as u32).len() as usize;
            },
        }

        return len;
    }

    pub fn encode<B: BufMut>(&self, buf: &mut B) -> Result<(), ()> {
        if self.encode_len() > buf.remaining_mut() { return Err(()); }

        VarInt::from_u32(self.purpose.code() as u32).write(buf)?;

        match self.purpose {
            DatagramPurpose::Stardust { channel } => {
                VarInt::from_u32(channel.into()).write(buf)?;
            },
        }

        todo!()
    }

    pub fn decode<B: Buf>(&self, buf: &mut B) -> Result<Self, ()> {
        todo!()
    }
}

impl DatagramPurpose {
    fn code(&self) -> DatagramPurposeCode {
        match self {
            DatagramPurpose::Stardust { channel: _ } => DatagramPurposeCode::Stardust,
        }
    }
}
