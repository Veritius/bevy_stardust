use std::collections::{BTreeMap, VecDeque};
use bytes::Bytes;
use tracing::error;
use unbytes::Reader;
use crate::{connection::{packets::header::PacketHeaderFlags, reliability::{AckMemory, ReliabilityState, UnackedPacket}}, plugin::PluginConfiguration, sequences::SequenceId, varint::VarInt};
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
    pub reliability: &'a mut ReliabilityState,
    pub rel_packets: &'a mut BTreeMap<SequenceId, UnackedPacket>,
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

        // Run the parser function
        let reader = self.current.as_mut().unwrap();
        let result = parse_frame(reader);

        // Errors anywhere force us to discard the packet
        // Also, if we've finished reading it, clear the slot
        if result.is_err() || reader.remaining() == 0 {
            self.current = None;
        }

        // Return the result
        return Some(result);
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
    let iter = context.reliability.rec_ack(ack, ack_bits, ack_bits_len as u8);

    // An iterator is returned over all sequence IDs that have now been acked.
    // This is used to remove (free) any unacked packets in storage.
    // Since we don't want to return the iterator, we do this here instead.
    iter.for_each(|i| { context.rel_packets.remove(&i); });

    return Ok(())
}

fn parse_frame(
    reader: &mut Reader,
) -> Result<RecvFrame, PacketReadError> {
    // Get the byte for frame flags
    let flags: FrameFlags = reader.read_u8()
        .map_err(|_| PacketReadError::UnexpectedEnd)?
        .into();

    // Parse the frame header type
    let ftype = reader.read_u8()
        .map_err(|_| PacketReadError::UnexpectedEnd)?;
    let ftype = FrameType::try_from(ftype)
        .map_err(|_| PacketReadError::InvalidFrameType)?;

    // Get the frame channel id if present
    let ident = match flags.any_high(FrameFlags::IDENTIFIED) {
        false => None,
        true => Some(VarInt::read(reader)
            .map_err(|_| PacketReadError::InvalidFrameIdent)?),
    };

    // At the moment, all frame types require an associated ident
    // We check this here, but later on a better check must be added
    if ident.is_none() {
        return Err(PacketReadError::InvalidFrameIdent);
    }

    // Get the frame channel ordering if present
    let order = match flags.any_high(FrameFlags::ORDERED) {
        false => None,
        true => Some(reader.read_u16()
            .map_err(|_| PacketReadError::UnexpectedEnd)?.into()),
    };

    // There are extra constraints for Stardust messages
    if ftype == FrameType::Stardust {
        match ident {
            // Stardust messages only have 2^32 possible channels
            Some(x) if u64::from(x) > u32::MAX as u64 => {
                return Err(PacketReadError::InvalidFrameIdent);
            },

            // All checks passed, do nothing.
            _ => {},
        }
    }

    // Read the length of the packet
    let len: usize = VarInt::read(reader)
        .map_err(|_| PacketReadError::InvalidFrameLen)?
        .into();

    // Read the next few bytes as per len
    let payload = reader.read_bytes(len)
        .map_err(|_| PacketReadError::UnexpectedEnd)?;

    // Return the frame
    return Ok(RecvFrame { ftype, order, ident, payload });
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PacketReadError {
    UnexpectedEnd,
    InvalidFrameType,
    InvalidFrameIdent,
    InvalidFrameLen,
}