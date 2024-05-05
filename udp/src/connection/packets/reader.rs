use std::collections::VecDeque;
use bytes::Bytes;
use tracing::error;
use unbytes::Reader;
use crate::{connection::{packets::header::PacketHeaderFlags, reliability::{AckMemory, ReliablePackets}}, plugin::PluginConfiguration, sequences::SequenceId, varint::VarInt};
use super::frames::{FrameFlags, FrameType, RecvFrame};

/// Parses incoming packets into an iterator of `Frame` objects.
pub(crate) struct PacketReader {}

impl Default for PacketReader {
    fn default() -> Self {
        Self {

        }
    }
}

impl PacketReader {
    #[must_use]
    pub fn iter<'a>(&'a mut self, context: PacketReaderContext<'a>) -> PacketReaderIter<'a> {
        PacketReaderIter { inner: self, current: None, context }
    }
}

pub(crate) struct PacketReaderContext<'a> {
    pub queue: &'a mut VecDeque<Bytes>,
    pub config: &'a PluginConfiguration,
    pub reliability: &'a mut ReliablePackets,
}

/// Dropping this type may cause data loss.
/// Use [`is_safe_to_drop`](Self::is_safe_to_drop) to check if you can drop this without data loss.
pub(crate) struct PacketReaderIter<'a> {
    inner: &'a mut PacketReader,
    current: Option<Reader>,
    context: PacketReaderContext<'a>,
}

impl PacketReaderIter<'_> {
    #[inline]
    pub fn is_safe_to_drop(&self) -> bool {
        self.current.is_none()
    }
}

impl Drop for PacketReaderIter<'_> {
    fn drop(&mut self) {
        if !self.is_safe_to_drop() {
            error!("PacketReaderIter was dropped with unread data");
        }
    }
}

impl Iterator for PacketReaderIter<'_> {
    type Item = Result<RecvFrame, PacketReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        let new_reader = match self.current {
            Some(ref reader) => { reader.remaining() == 0 },
            None => true,
        };

        if new_reader {
            // Fetch the next message for reading.
            let bytes = self.context.queue.pop_front()?;
            let mut reader = Reader::new(bytes);

            // Read the first bit of information about the packet.
            if let Err(error) = parse_header(&mut reader, &mut self.context) {
                // If the header is broken, there's not much point to going further.
                return Some(Err(error));
            }

            // Set the reader variable
            self.current = Some(reader);
        };

        let reader = self.current.as_mut().unwrap();

        // Run the parser function
        Some(parse_frame(reader))
    }
}

fn parse_header(
    reader: &mut Reader,
    context: &mut PacketReaderContext,
) -> Result<(), PacketReadError> {
    // Read the packet header flags byte
    let flags = PacketHeaderFlags(reader.read_byte()
        .map_err(|_| PacketReadError::UnexpectedEnd)?);

    // If the packet is flagged reliable, it has a sequence id
    if flags.any_high(PacketHeaderFlags::RELIABLE) {
        let seq = SequenceId(reader.read_u16()
            .map_err(|_| PacketReadError::UnexpectedEnd)?);
        context.reliability.ack_seq(seq);
    }

    // These reliability values are always present
    let ack_bits_len = context.config.reliable_bitfield_length;
    let ack = SequenceId(reader.read_u16()
        .map_err(|_| PacketReadError::UnexpectedEnd)?);
    let ack_bits = AckMemory::from_slice(reader.read_slice(ack_bits_len)
        .map_err(|_| PacketReadError::UnexpectedEnd)?).unwrap();
    context.reliability.rec_ack(ack, ack_bits, ack_bits_len as u8);

    return Ok(())
}

fn parse_frame(
    reader: &mut Reader,
) -> Result<RecvFrame, PacketReadError> {
    // Get the byte for frame flags
    let flags: FrameFlags = reader.read_u8()
        .map_err(|_| PacketReadError::UnexpectedEnd)?
        .into();

    // Get the frame channel id if present
    let ident = match flags.any_high(FrameFlags::IDENTIFIED) {
        false => None,
        true => Some(VarInt::read(reader)
            .map_err(|_| PacketReadError::UnexpectedEnd)?),
    };

    // Get the frame channel ordering if present
    let order = match flags.any_high(FrameFlags::ORDERED) {
        false => None,
        true => Some(reader.read_u16()
            .map_err(|_| PacketReadError::UnexpectedEnd)?.into()),
    };

    // Parse the frame header type
    let ftype = reader.read_u8()
        .map_err(|_| PacketReadError::UnexpectedEnd)?;
    let ftype = FrameType::try_from(ftype)
        .map_err(|_| PacketReadError::InvalidFrameType)?;

    // Read the length of the packet
    let len: usize = VarInt::read(reader)
        .map_err(|_| PacketReadError::InvalidFrameLen)?
        .into();

    // Read the next few bytes as per len
    let payload = reader.read_bytes(len)
        .map_err(|_| PacketReadError::UnexpectedEnd)?;

    // Return the frame
    return Ok(RecvFrame { flags, ftype, order, ident, payload });
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PacketReadError {
    UnexpectedEnd,
    InvalidFrameType,
    InvalidFrameLen,
}