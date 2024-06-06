use std::ops::{BitOr, BitOrAssign, BitAnd, BitAndAssign};
use bytes::BufMut;
use unbytes::{EndOfInput, Reader};
use crate::{connection::reliability::AckMemory, sequences::SequenceId};

pub(super) enum PacketHeader {
    Reliable {
        seq: SequenceId,
        ack: SequenceId,
        bits: AckMemory,
        len: u8,
    },

    Unreliable,
}

impl PacketHeader {
    pub fn read(reader: &mut Reader) -> Result<Self, PacketReadError> {
        todo!()
    }

    pub fn write<B: BufMut>(mut b: B) {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PacketReadError {
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