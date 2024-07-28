use bevy_stardust::prelude::ChannelId;
use bevy_stardust_extras::numbers::{Sequence, VarInt};
use bytes::{Buf, BufMut, Bytes};
use crate::Connection;

impl Connection {
    /// Call when a datagram is received.
    pub fn recv_dgram(&mut self, dgram: Bytes) {
        todo!()
    }
}

pub(super) enum DatagramHeader {
    Stardust {
        channel: ChannelId,
    },

    StardustSequenced {
        channel: ChannelId,
        sequence: Sequence<u16>,
    },
}

impl DatagramHeader {
    pub fn read<B: Buf>(buf: &mut B) -> Result<DatagramHeader, ()> {
        let code: u64 = VarInt::read(buf)?.into();

        match code {
            0 => {
                let channel = VarInt::read(buf)
                    .and_then(|v| u32::try_from(v))
                    .map(|v| ChannelId::from(v))?;

                return Ok(DatagramHeader::Stardust {
                    channel,
                });
            },

            1 => {
                let channel = VarInt::read(buf)
                    .and_then(|v| u32::try_from(v))
                    .map(|v| ChannelId::from(v))?;

                let sequence = match buf.remaining() >= 2 {
                    true => Sequence::from(buf.get_u16()),
                    false => return Err(()),
                };

                return Ok(DatagramHeader::StardustSequenced {
                    channel,
                    sequence,
                });
            },

            _ => return Err(()),
        }
    }

    pub fn write<B: BufMut>(&self, buf: &mut B) -> Result<(), ()> {
        match self {
            DatagramHeader::Stardust { channel } => {
                VarInt::from_u32(0).write(buf)?;
                VarInt::from_u32((*channel).into()).write(buf)?;
            },

            DatagramHeader::StardustSequenced { channel, sequence } => {
                VarInt::from_u32(1).write(buf)?;
                VarInt::from_u32((*channel).into()).write(buf)?;

                match buf.remaining_mut() >= 2 {
                    true => buf.put_u16(sequence.inner()),
                    false => return Err(()),
                }
            }
        }

        return Ok(());
    }
}

pub(crate) struct DatagramSequences(Sequence<u16>);

impl DatagramSequences {
    pub fn new() -> Self {
        Self(Sequence::default())
    }
}