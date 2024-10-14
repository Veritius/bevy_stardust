use std::collections::BTreeMap;
use bevy_stardust::prelude::{ChannelId, ChannelMessage, Message};
use bevy_stardust_extras::numbers::{Sequence, VarInt};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::{Connection, ConnectionEvent};

pub(crate) struct DatagramBuilder<'a> {
    pub header: DatagramHeader,
    pub payload: &'a [u8],
}

impl<'a> DatagramBuilder<'a> {
    pub fn build<B: BufMut>(self, buf: &mut B) -> Result<usize, ()> {
        // Make sure that the buffer has enough space for writing
        let size = self.header.size() + self.payload.len();
        if size > buf.remaining_mut() { return Err(()); }

        // Write to the buffer
        self.header.write(buf).unwrap();
        buf.put(self.payload);

        // Success
        return Ok(size);
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
        let header = match DatagramHeader::read(&mut payload) {
            Ok(v) => v,
            Err(_) => return,
        };

        match header {
            DatagramHeader::Stardust { channel } => {
                self.shared.events.push(ConnectionEvent::ReceivedMessage(ChannelMessage {
                    channel,
                    message: Message::from_bytes(payload),
                }));
            },

            DatagramHeader::StardustSequenced { channel, sequence } => {
                let seq = self.incoming_datagrams.sequences.entry(channel)
                    .or_insert_with(|| IncomingDatagramSequence::new());

                if seq.latest(sequence) {
                    self.shared.events.push(ConnectionEvent::ReceivedMessage(ChannelMessage {
                        channel,
                        message: Message::from_bytes(payload),
                    }));
                }
            },
        }
    }

    pub(crate) fn channel_dgram_out_seq(&mut self, channel: ChannelId) -> &mut OutgoingDatagramSequence {
        self.outgoing_datagrams.sequences.entry(channel)
            .or_insert_with(|| OutgoingDatagramSequence::new())
    }

    /// Try to send a datagram, returns `false` if the size exceeds `size_limit`.
    pub(crate) fn try_send_dgram(
        &mut self,
        size_limit: usize,
        header: DatagramHeader,
        payload: Bytes,
    ) -> bool {
        // Create the datagram header
        let size = header.size() + payload.len();

        // Check if it can be sent as a datagram
        // If it can't, we have to return an error
        if size > size_limit { return false; }

        // We have to put this in a new allocation as most QUIC implementations
        // will only take a single slice/Bytes for a datagram payload
        // This shouldn't panic, since BytesMut grows to fit
        let mut newbuf = BytesMut::with_capacity(size);
        header.write(&mut newbuf).unwrap();
        newbuf.extend_from_slice(&payload);

        // Also check that the sizes are correct
        debug_assert_eq!(size, newbuf.len());

        // Queue the datagram for transmission by the QUIC implementation
        self.shared.events.push(crate::ConnectionEvent::TransmitDatagram(newbuf.freeze()));

        // Success
        return true;
    }

    pub(crate) fn send_dgram_wrap_on_fail(
        &mut self,
        size_limit: usize,
        header: DatagramHeader,
        payload: Bytes,
    ) {
        // Try to send a datagram normally
        if self.try_send_dgram(size_limit, header.clone(), payload.clone()) { return };

        // On failure, send it wrapped inside a stream
        self.outgoing_streams_handle().send_wrapped_dgram_chunks([header.alloc(), payload].into_iter());
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