use std::collections::VecDeque;
use bytes::Bytes;
use tracing::error;
use unbytes::Reader;
use crate::{connection::reliability::ReliablePackets, plugin::PluginConfiguration};
use super::{frames::FrameReadError, header::{PacketHeader, PacketReadError}};
use super::frames::RecvFrame;

/// Parses incoming packets into an iterator of `Frame` objects.
pub(crate) struct PacketParser {}

impl Default for PacketParser {
    fn default() -> Self {
        Self {}
    }
}

impl PacketParser {
    #[must_use]
    pub fn iter<'a>(&'a mut self, context: PacketReaderContext<'a>) -> PacketReaderIter<'a> {
        PacketReaderIter { _inner: self, current: None, context }
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
    _inner: &'a mut PacketParser,
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
    type Item = Result<RecvFrame, ParsingError>;

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
) -> Result<(), ParsingError> {
    let bitlen = context.config.reliable_bitfield_length;
    let bitlen8: u8 = bitlen.try_into().unwrap();

    match PacketHeader::read(reader, bitlen) {
        Ok(PacketHeader::Reliable { seq, ack, bits }) => {
            context.reliability.ack_seq(seq);
            context.reliability.rec_ack(ack, bits, bitlen8);
            Ok(())
        },

        Ok(PacketHeader::Unreliable { ack, bits }) => {
            context.reliability.rec_ack(ack, bits, bitlen8);
            Ok(())
        },

        Err(err) => Err(ParsingError::PacketError(err)),
    }
}

fn parse_frame(
    reader: &mut Reader,
) -> Result<RecvFrame, ParsingError> {
    match RecvFrame::read(reader) {
        Ok(frame) => Ok(frame),
        Err(err) => Err(ParsingError::FrameError(err)),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParsingError {
    PacketError(PacketReadError),
    FrameError(FrameReadError),
}