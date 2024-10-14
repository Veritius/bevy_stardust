use std::collections::BTreeMap;
use bevy_stardust::prelude::{ChannelId, ChannelMessage};
use bevy_stardust_extras::numbers::{Sequence, VarInt};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::{Connection, ConnectionEvent};

pub(crate) struct DatagramBuilder<'a> {
    pub header: DatagramHeader,
    pub payload: &'a [u8],
}

impl<'a> DatagramBuilder<'a> {
    pub fn size(&self) -> usize {
        self.header.size() + self.payload.len()
    }

    pub fn write<B: BufMut>(self, buf: &mut B) -> Result<(), ()> {
        // Make sure that the buffer has enough space for writing
        if self.size() > buf.remaining_mut() { return Err(()); }

        // Write to the buffer
        self.header.write(buf).unwrap();
        buf.put(self.payload);

        // Success
        return Ok(());
    }
}

pub(crate) struct DatagramReceive {
    pub header: DatagramHeader,
    pub payload: Bytes,
}

impl DatagramReceive {
    pub fn parse<B: Buf>(buf: &mut B) -> Result<Self, ()> {
        let header = match DatagramHeader::read(buf) {
            Ok(v) => v,
            Err(_) => return Err(()),
        };

        return Ok(DatagramReceive {
            header,
            payload: buf.copy_to_bytes(buf.remaining()),
        });
    }
}

impl Connection {
    /// Call when a datagram is received.
    pub fn recv_dgram(&mut self, mut payload: Bytes) {
        match DatagramReceive::parse(&mut payload) {
            Ok(dgram) => match dgram.header {
                DatagramHeader::Stardust { channel } => {
                    // Store the event in the message queue
                    self.shared.events.push(ConnectionEvent::ReceivedMessage(ChannelMessage {
                        channel,
                        message: dgram.payload.into(),
                    }));
                },

                DatagramHeader::StardustSequenced { channel, sequence: remote } => {
                    // Fetch the current local sequence value
                    let local = self.incoming_datagrams.sequences.entry(channel)
                        .or_insert_with(|| IncomingDatagramSequence::new());

                    // Check that the message isn't old
                    if !local.latest(remote) {
                        todo!()
                    }

                    // Store the event in the message queue
                    self.shared.events.push(ConnectionEvent::ReceivedMessage(ChannelMessage {
                        channel,
                        message: dgram.payload.into(),
                    }));
                },
            },

            Err(_) => todo!(),
        }
    }

    pub(crate) fn send_dgram(
        &mut self,
        datagram: DatagramBuilder,
        size_limit: usize,
    ) {
        // Check the datagram is of an acceptable size
        let size = datagram.size();
        if size > size_limit {
            // Send it over a stream so it gets fragmented
            self.send_dgram_over_stream(datagram);
            return;
        }

        // Allocate space for the datagram and write it
        let mut buf = BytesMut::with_capacity(size);
        datagram.write(&mut buf);
        let buf = buf.freeze();

        // If this isn't the case, we allocated incorrectly
        debug_assert_eq!(size, buf.len());

        // Save the datagram
        self.shared.events.push(ConnectionEvent::TransmitDatagram(buf));
    }

    pub(crate) fn channel_dgram_out_seq(&mut self, channel: ChannelId) -> &mut OutgoingDatagramSequence {
        self.outgoing_datagrams.sequences.entry(channel)
            .or_insert_with(|| OutgoingDatagramSequence::new())
    }
}

pub(crate) struct IncomingDatagrams {
    sequences: BTreeMap<ChannelId, IncomingDatagramSequence>,
}

impl IncomingDatagrams {
    pub fn new() -> Self {
        Self {
            sequences: BTreeMap::new(),
        }
    }
}

pub(crate) struct OutgoingDatagrams {
    sequences: BTreeMap<ChannelId, OutgoingDatagramSequence>,
}

impl OutgoingDatagrams {
    pub fn new() -> Self {
        Self {
            sequences: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Copy)]
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

    pub fn alloc(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(8);
        self.write(&mut buf).unwrap();
        return buf.freeze();
    }

    pub fn size(&self) -> usize {
        let mut tally = 0;

        match self {
            DatagramHeader::Stardust { channel } => {
                tally += VarInt::len_u32(0) as usize;
                tally += VarInt::len_u32((*channel).into()) as usize;
            },

            DatagramHeader::StardustSequenced { channel, sequence: _ } => {
                tally += VarInt::len_u32(1) as usize;
                tally += VarInt::len_u32((*channel).into()) as usize;
                tally += 2; // sequence value is always 2 bytes
            },
        }

        return tally;
    }
}

#[derive(Debug)]
pub(crate) struct IncomingDatagramSequence(Sequence<u16>);

impl IncomingDatagramSequence {
    pub fn new() -> Self {
        Self(Sequence::from(u16::MAX))
    }

    pub fn latest(&mut self, index: Sequence<u16>) -> bool {
        if index > self.0 {
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
        Self(Sequence::from(0))
    }

    pub fn next(&mut self) -> Sequence<u16> {
        let v = self.0;
        self.0.increment();
        return v;
    }
}