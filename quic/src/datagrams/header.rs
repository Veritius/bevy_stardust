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

        // The purpose code is always present
        len += VarInt::len_u32(self.purpose.code() as u32) as usize;

        match self.purpose {
            DatagramPurpose::Stardust { channel } => { len += VarInt::len_u32(channel.into()) as usize; },
        }

        return len;
    }

    pub fn encode<B: BufMut>(&self, buf: &mut B) -> Result<(), ()> {
        // Check we have enough space to encode this
        if self.encode_len() > buf.remaining_mut() { return Err(()); }

        // The purpose code always prefixes the header
        VarInt::from_u32(self.purpose.code() as u32).write(buf)?;

        match self.purpose {
            DatagramPurpose::Stardust { channel } => { VarInt::from_u32(channel.into()).write(buf)?; },
        }

        return Ok(())
    }

    pub fn decode<B: Buf>(&self, buf: &mut B) -> Result<Self, ()> {
        // Get the purpose code prefixing the datagram
        let code = VarInt::read(buf)
            .and_then(u32::try_from)
            .map_err(|_| ())?
            .try_into()?;

        // Now that we know the code, we can get the fields
        let purpose = match code {
            DatagramPurposeCode::Stardust => DatagramPurpose::Stardust {
                channel: VarInt::read(buf).and_then(u32::try_from).map_err(|_| ())?.into(),
            },
        };

        // Return the decoded header
        return Ok(DatagramHeader { purpose });
    }
}

impl DatagramPurpose {
    fn code(&self) -> DatagramPurposeCode {
        match self {
            DatagramPurpose::Stardust { channel: _ } => DatagramPurposeCode::Stardust,
        }
    }
}

impl TryFrom<u32> for DatagramPurposeCode {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        return Ok(match value {
            0 => Self::Stardust,

            _ => return Err(()),
        });
    }
}