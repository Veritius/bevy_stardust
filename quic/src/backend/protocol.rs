use std::{collections::BTreeMap, sync::Arc};
use bevy_stardust::{channels::ChannelRegistry, prelude::ChannelId};
use bevy_stardust_extras::numbers::{VarInt, Sequence};
use bytes::{Buf, BufMut, Bytes};
use quinn_proto::StreamId;

pub(super) struct Protocol {
    channels: Arc<ChannelRegistry>,

    seq_map: SeqMap,

    send_stream_id_index: u64,
    send_stream_id_map: BTreeMap<ChannelId, StreamId>,
    send_stream_id_map_rev: BTreeMap<StreamId, ChannelId>,
    send_stream_waiting_chunks: BTreeMap<StreamId, Vec<Bytes>>,

    recv_stream_buffers: BTreeMap<StreamId, RecvStreamBuf>,
}

impl Protocol {
    pub fn new(channels: Arc<ChannelRegistry>) -> Protocol {
        Protocol {
            channels,

            seq_map: SeqMap::new(),

            send_stream_id_index: 0,
            send_stream_id_map: BTreeMap::new(),
            send_stream_id_map_rev: BTreeMap::new(),
            send_stream_waiting_chunks: BTreeMap::new(),

            recv_stream_buffers: BTreeMap::new(),
        }
    }
}

struct RecvStreamBuf {

}

struct SeqMap {
    local: BTreeMap<ChannelId, SeqRecord>,
    remote: BTreeMap<ChannelId, SeqRecord>,
}

impl SeqMap {
    fn new() -> Self {
        Self {
            local: BTreeMap::new(),
            remote: BTreeMap::new(),
        }
    }
}

struct SeqRecord(Sequence<u16>);

impl SeqRecord {
    fn new() -> Self {
        Self(Sequence::from(0))
    }

    fn next(&mut self) -> Sequence<u16> {
        let v = self.0;
        self.0.increment();
        return v;
    }

    fn latest(&mut self, index: Sequence<u16>) -> bool {
        if index > self.0 {
            self.0 = index;
            return true;
        } else {
            return false;
        }
    }
}

struct Segment(Bytes);

impl Segment {
    fn read<B: Buf>(buf: &mut B) -> Result<Self, EndOfInput> {
        let len: u64 = VarInt::read(buf).map_err(|_| EndOfInput)?.into();
        let len = usize::try_from(len).map_err(|_| EndOfInput)?;
        if buf.remaining() < len { return Err(EndOfInput) }
        return Ok(Segment(buf.copy_to_bytes(len)));
    }

    fn write<B: BufMut>(&self, buf: &mut B) -> Result<(), InsufficientSpace> {
        // Encode the length of the segment as a variable length integer
        // Unwrapping here is probably fine because I doubt anyone
        // is going to send a 46116 terabyte message...
        VarInt::try_from(self.0.len() as u64).unwrap()
            .write(buf).map_err(|_| InsufficientSpace)?;

        if buf.remaining_mut() < self.0.len() { return Err(InsufficientSpace) }

        // Put the rest of the segment into the buffer
        buf.put_slice(&self.0);

        // Done.
        return Ok(());
    }
}

enum Message {
    Unordered {
        channel: ChannelId,
        payload: Bytes,
    },

    Sequenced {
        channel: ChannelId,
        sequence: Sequence<u16>,
        payload: Bytes,
    },
}

impl Message {
    fn read<B: Buf>(buf: &mut B) -> Result<Self, MessageDecodeError> {
        let code: u64 = decode_varint(buf)?.into();

        match code {
            0 => {
                return Ok(Self::Unordered {
                    channel: decode_varint(buf)?.into(),
                    payload: Segment::read(buf)?.0,
                })
            },

            1 => {
                return Ok(Self::Sequenced {
                    channel: decode_varint(buf)?.into(),
                    sequence: {
                        if buf.remaining() < 2 { return Err(MessageDecodeError::EndOfInput); }
                        buf.get_u16().into()
                    },
                    payload: Segment::read(buf)?.0,
                })
            },

            _ => return Err(MessageDecodeError::UnknownCode),
        }
    }

    fn write<B: BufMut>(&self, buf: &mut B) -> Result<(), InsufficientSpace> {
        match self {
            Message::Unordered {
                channel,
                payload,
            } => {
                encode_varint(0, buf)?;
                encode_varint(*channel, buf)?;

                Segment(payload.clone()).write(buf)?;
            },

            Message::Sequenced {
                channel,
                sequence,
                payload,
            } => {
                encode_varint(1, buf)?;
                encode_varint(*channel, buf)?;

                if buf.remaining_mut() < 2 { return Err(InsufficientSpace) }
                buf.put_u16(sequence.inner());

                Segment(payload.clone()).write(buf)?;
            },
        }

        return Ok(());
    }
}

enum MessageDecodeError {
    UnknownCode,
    EndOfInput,
    TooLarge,
}

impl From<EndOfInput> for MessageDecodeError {
    fn from(value: EndOfInput) -> Self {
        Self::EndOfInput
    }
}

struct EndOfInput;
struct InsufficientSpace;

fn decode_varint<B: Buf>(buf: &mut B) -> Result<VarInt, EndOfInput> {
    VarInt::read(buf).map_err(|_| EndOfInput)
}

fn encode_varint<B: BufMut>(value: impl Into<VarInt>, buf: &mut B) -> Result<(), InsufficientSpace> {
    value.into().write(buf).map_err(|_| InsufficientSpace)
}