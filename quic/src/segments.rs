use bevy_stardust::prelude::{ChannelId, ChannelMessage};
use bevy_stardust_extras::numbers::{Sequence, VarInt};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::{Connection, ConnectionEvent, MessageSequence};

pub(crate) struct Segment {
    pub header: Header,
    pub payload: Bytes,
}

impl Segment {
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

    pub fn parse<B: Buf>(buf: &mut B) -> Result<Self, ()> {
        let header = match Header::read(buf) {
            Ok(v) => v,
            Err(_) => return Err(()),
        };

        return Ok(Segment {
            header,
            payload: buf.copy_to_bytes(buf.remaining()),
        });
    }
}

impl Connection {
    /// Call when a datagram is received.
    pub fn recv_dgram(&mut self, mut payload: Bytes) {
        match Segment::parse(&mut payload) {
            Ok(segment) => self.recv_segment(segment),
            Err(_) => { self.shared.heat.increase(16); },
        }
    }

    pub(crate) fn recv_segment(&mut self, segment: Segment) {
        match segment.header {
            Header::UnorderedMessage { channel } => {
                // Store the event in the message queue
                self.shared.events.push(ConnectionEvent::ReceivedMessage(ChannelMessage {
                    channel,
                    message: segment.payload.into(),
                }));
            },

            Header::SequencedMessage { channel, sequence } => {
                // Fetch the current local sequence value
                let remote = self.message_sequences.remote.entry(channel)
                    .or_insert_with(|| MessageSequence::new());

                // Check that the message isn't old
                if remote.latest(sequence) {
                    return; // Discard it
                }

                // Store the event in the message queue
                self.shared.events.push(ConnectionEvent::ReceivedMessage(ChannelMessage {
                    channel,
                    message: segment.payload.into(),
                }));
            },
        }
    }

    pub(crate) fn send_segment(
        &mut self,
        chunk: Segment,
        size_limit: usize,
    ) {
        // Check the chunk isn't oversize
        let size = chunk.size();
        if size > size_limit {
            // Send it over a stream so it gets fragmented
            self.stream_segment_transient(chunk);
            return;
        }

        // Allocate space for the chunk and write it
        let mut buf = BytesMut::with_capacity(size);
        chunk.write(&mut buf).unwrap();
        let buf = buf.freeze();

        // If this isn't the case, we allocated incorrectly
        debug_assert_eq!(size, buf.len());

        // Save the datagram
        self.shared.events.push(ConnectionEvent::TransmitDatagram(buf));
    }
}

#[derive(Clone, Copy)]
pub(super) enum Header {
    UnorderedMessage {
        channel: ChannelId,
    },

    SequencedMessage {
        channel: ChannelId,
        sequence: Sequence<u16>,
    },
}

impl Header {
    pub fn read<B: Buf>(buf: &mut B) -> Result<Header, ()> {
        let code: u64 = VarInt::read(buf)?.into();

        match code {
            0 => {
                let channel = VarInt::read(buf)
                    .and_then(|v| u32::try_from(v))
                    .map(|v| ChannelId::from(v))?;

                return Ok(Header::UnorderedMessage {
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

                return Ok(Header::SequencedMessage {
                    channel,
                    sequence,
                });
            },

            _ => return Err(()),
        }
    }

    pub fn write<B: BufMut>(&self, buf: &mut B) -> Result<(), ()> {
        match self {
            Header::UnorderedMessage { channel } => {
                VarInt::from_u32(0).write(buf)?;
                VarInt::from_u32((*channel).into()).write(buf)?;
            },

            Header::SequencedMessage { channel, sequence } => {
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
            Header::UnorderedMessage { channel } => {
                tally += VarInt::len_u32(0) as usize;
                tally += VarInt::len_u32((*channel).into()) as usize;
            },

            Header::SequencedMessage { channel, sequence: _ } => {
                tally += VarInt::len_u32(1) as usize;
                tally += VarInt::len_u32((*channel).into()) as usize;
                tally += 2; // sequence value is always 2 bytes
            },
        }

        return tally;
    }
}