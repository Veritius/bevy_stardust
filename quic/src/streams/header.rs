use bevy_stardust::prelude::*;
use bevy_stardust_extras::numbers::VarInt;
use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy)]
pub(super) struct StreamHeader {
    pub purpose: StreamPurpose,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum StreamPurpose {
    Stardust { channel: ChannelId },
    Datagram,
}

#[derive(Clone, Copy)]
#[repr(u32)]
enum StreamPurposeCode {
    Stardust = 0,
    Datagram = 1,
}

impl StreamHeader {
    pub fn encode_len(&self) -> usize {
        let mut len: usize = 0;

        // The purpose code is always present
        len += VarInt::len_u32(self.purpose.code() as u32) as usize;

        match self.purpose {
            StreamPurpose::Stardust { channel } => { len += VarInt::len_u32(channel.into()) as usize; },
            StreamPurpose::Datagram => {},
        }

        return len;
    }

    pub fn encode<B: BufMut>(&self, buf: &mut B) -> Result<(), ()> {
        // Check we have enough space to encode this
        if self.encode_len() > buf.remaining_mut() { return Err(()); }

        // The purpose code always prefixes the header
        VarInt::from_u32(self.purpose.code() as u32).write(buf)?;

        match self.purpose {
            StreamPurpose::Stardust { channel } => { VarInt::from_u32(channel.into()).write(buf)?; },

            StreamPurpose::Datagram => {},
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
            StreamPurposeCode::Stardust => StreamPurpose::Stardust {
                channel: VarInt::read(buf).and_then(u32::try_from).map_err(|_| ())?.into(),
            },

            StreamPurposeCode::Datagram => StreamPurpose::Datagram,
        };

        // Return the decoded header
        return Ok(StreamHeader { purpose });
    }
}

impl StreamPurpose {
    fn code(&self) -> StreamPurposeCode {
        match self {
            StreamPurpose::Stardust { channel: _ } => StreamPurposeCode::Stardust,
            StreamPurpose::Datagram => StreamPurposeCode::Datagram,
        }
    }

    #[inline]
    pub fn is_framed(&self) -> bool {
        self.code().is_framed()
    }
}

impl TryFrom<u32> for StreamPurposeCode {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        return Ok(match value {
            0 => Self::Stardust,
            1 => Self::Datagram,

            _ => return Err(()),
        });
    }
}

impl StreamPurposeCode {
    fn is_framed(&self) -> bool {
        match self {
            StreamPurposeCode::Stardust => true,
            StreamPurposeCode::Datagram => false,
        }
    }
}