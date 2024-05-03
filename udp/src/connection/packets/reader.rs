use std::collections::VecDeque;
use bytes::Bytes;
use tracing::error;
use unbytes::Reader;
use crate::{connection::{packets::header::PacketHeaderFlags, reliability::ReliablePackets}, plugin::PluginConfiguration, sequences::SequenceId};
use super::frames::RecvFrame;

/// Parses incoming packets into an iterator of `Frame` objects.
pub(crate) struct PacketReader {
    queue: VecDeque<Bytes>,
}

impl Default for PacketReader {
    fn default() -> Self {
        Self {
            queue: VecDeque::with_capacity(16),
        }
    }
}

impl PacketReader {
    pub fn iter<'a>(&'a mut self, context: PacketReaderContext<'a>) -> PacketReaderIter<'a> {
        PacketReaderIter { inner: self, current: None, context }
    }

    pub(in crate::connection) fn push(
        &mut self,
        packet: Bytes,
    ) {
        self.queue.push_back(packet)
    }
}

pub(crate) struct PacketReaderContext<'a> {
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
        // Create reader if none is present
        // This occurs when this type is first created
        // or if the previous frame was consumed
        let reader = match self.current {
            Some(ref mut reader) => reader,
            None => {
                // Fetch the next message for reading.
                let bytes = self.inner.queue.pop_front()?;
                let mut reader = Reader::new(bytes);

                // Read the first bit of information about the packet.
                if let Err(error) = parse_header(&mut reader, &mut self.context) {
                    // If the header is broken, there's not much point to going further.
                    return Some(Err(error));
                }

                // SAFETY: It's assigned and then immediately accessed.
                // I have to do this because get_or_insert_with doesn't
                // allow you to terminate the outer function.
                // TODO: Find a safe solution.
                self.current = Some(reader);
                unsafe { self.current.as_mut().unwrap_unchecked() }
            },
        };

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
        .map_err(|_| PacketReadError::InvalidHeader)?);

    // If the packet is flagged reliable, it has a sequence id
    if flags.any_high(PacketHeaderFlags::RELIABLE) {
        let seq = SequenceId(reader.read_u16()
            .map_err(|_| PacketReadError::InvalidHeader)?);
        context.reliability.ack_seq(seq);
    }

    // These reliability values are always present
    let ack = SequenceId(reader.read_u16()
        .map_err(|_| PacketReadError::InvalidHeader)?);
    let mut ack_bits = [0u8; 16];
    // SequenceId(reader.read_slice(context.config.reliable_bitfield_length)
    //     .map_err(|_| PacketReadError::InvalidHeader)?);

    return Ok(())
}

fn parse_frame(
    reader: &mut Reader,
) -> Result<RecvFrame, PacketReadError> {
    todo!()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PacketReadError {
    InvalidHeader,
}