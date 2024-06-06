use std::ops::{BitOr, BitOrAssign, BitAnd, BitAndAssign};
use bytes::BufMut;
use unbytes::{EndOfInput, Reader};
use crate::{connection::reliability::AckMemory, sequences::SequenceId};

pub(super) enum PacketHeader {
    Reliable {
        seq: SequenceId,
        ack: SequenceId,
        bits: AckMemory,
    },

    Unreliable,
}

impl PacketHeader {
    pub fn read(reader: &mut Reader, bitlen: usize) -> Result<Self, PacketReadError> {
        // Read the packet header flags byte
        let flags = PacketHeaderFlags(reader.read_byte()
            .map_err(|_| PacketReadError::UnexpectedEnd)?);

        // Some data for reading things
        let is_reliable = flags & PacketHeaderFlags::RELIABLE > 0;

        if is_reliable {
            // If the packet is flagged reliable, it has a sequence id
            let seq = if is_reliable {
                SequenceId(reader.read_u16()
                    .map_err(|_| PacketReadError::UnexpectedEnd)?)
            } else { SequenceId::new(0) }; // default

            // These reliability values are always present
            let ack = SequenceId(reader.read_u16()
                .map_err(|_| PacketReadError::UnexpectedEnd)?);
            let bits = AckMemory::from_slice(reader.read_slice(bitlen)
                .map_err(|_| PacketReadError::UnexpectedEnd)?).unwrap();

            // Return the packet, we have everything
            return Ok(Self::Reliable { seq, ack, bits });
        } else {
            // Unreliable packets need no further work
            return Ok(Self::Unreliable);
        }
    }

    pub fn write<B: BufMut>(&self, mut b: B, bitlen: usize) {
        match self {
            PacketHeader::Reliable { seq, ack, bits } => {
                let flags = PacketHeaderFlags::RELIABLE;
                b.put_u8(flags.0);

                b.put_u16(seq.0);
                b.put_u16(ack.0);

                let arr = bits.into_array();
                b.put(&arr[..bitlen]);
            },

            PacketHeader::Unreliable => {
                let flags = PacketHeaderFlags::EMPTY;
                b.put_u8(flags.0);
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PacketReadError {
    UnexpectedEnd,
}

impl From<EndOfInput> for PacketReadError {
    fn from(value: EndOfInput) -> Self {
        Self::UnexpectedEnd
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct PacketHeaderFlags(pub u8);

impl PacketHeaderFlags {
    pub const EMPTY: Self = Self(0);

    pub const RELIABLE: Self = Self(1 << 0);
}

impl BitOr for PacketHeaderFlags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for PacketHeaderFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 = self.0 | rhs.0;
    }
}

impl BitAnd for PacketHeaderFlags {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for PacketHeaderFlags {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 = self.0 & rhs.0;
    }
}

impl PartialEq<u8> for PacketHeaderFlags {
    fn eq(&self, other: &u8) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<u8> for PacketHeaderFlags {
    fn partial_cmp(&self, other: &u8) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}