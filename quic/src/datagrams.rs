use bevy_stardust::prelude::{ChannelId, ChannelMessage, Message};
use bevy_stardust_extras::numbers::{Sequence, VarInt};
use bytes::{Buf, BufMut, Bytes};
use crate::{Connection, ConnectionEvent};

impl Connection {
    /// Sets the max size of datagrams that will be output.
    /// Fails if `size` is less than `1200`, which is below the QUIC minimum.
    /// 
    /// Defaults to `1200`, the QUIC minimum.
    pub fn set_dgram_max_size(&mut self, size: usize) -> Result<(), ()> {
        if size < 1200 { return Err(()) }
        self.datagram_max_size = size;
        return Ok(())
    }

    /// Call when a datagram is received.
    pub fn recv_dgram(&mut self, mut payload: Bytes) {
        let header = match DatagramHeader::read(&mut payload) {
            Ok(v) => v,
            Err(_) => return,
        };

        match header {
            DatagramHeader::Stardust { channel } => {
                self.events.push_back(ConnectionEvent::ReceivedMessage(ChannelMessage {
                    channel,
                    payload: Message::from_bytes(payload),
                }));
            },

            DatagramHeader::StardustSequenced { channel, sequence } => {
                let seq = self.incoming_datagram_channel_sequences.entry(channel)
                    .or_insert_with(|| IncomingDatagramSequence::new());

                if seq.latest(sequence) {
                    self.events.push_back(ConnectionEvent::ReceivedMessage(ChannelMessage {
                        channel,
                        payload: Message::from_bytes(payload),
                    }));
                }
            },
        }
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

#[derive(Debug)]
pub(crate) struct IncomingDatagramSequence(Sequence<u16>);

impl IncomingDatagramSequence {
    pub fn new() -> Self {
        Self(Sequence::default())
    }

    fn latest(&mut self, index: Sequence<u16>) -> bool {
        if self.0 > index {
            self.0 = index;
            return true;
        } else {
            return false;
        }
    }
}

#[derive(Debug)]
pub(crate) struct OutgoingDatagramSequence(Sequence<u16>);

impl OutgoingDatagramSequence {
    pub fn new() -> Self {
        Self(Sequence::default())
    }

    fn next(&mut self) -> Sequence<u16> {
        todo!()
    }
}