use std::ops::{BitOr, BitOrAssign};

use bytes::Bytes;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct PacketHeader(pub u16);

impl PacketHeader {
    pub const FLAG_RELIABLE: Self = Self(1);

    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }
}

impl BitOr for PacketHeader {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for PacketHeader {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.bitor(rhs);
    }
}

impl From<u16> for PacketHeader {
    #[inline]
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<PacketHeader> for u16 {
    #[inline]
    fn from(value: PacketHeader) -> Self {
        value.0
    }
}

/// Management frame types, with an `Ord` implementation comparing how important it is that the frame is sent.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(super) enum PacketFrameId {
    Padding = 0,
    Ping = 1,
}

impl TryFrom<u8> for PacketFrameId {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use PacketFrameId::*;
        Ok(match value {
            0 => Padding,
            1 => Ping,
            _ => { return Err(()) }
        })
    }
}

pub(super) struct PacketFrame {
    pub id: PacketFrameId,
    pub pld: Bytes,
}

impl std::fmt::Debug for PacketFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PacketFrame")
        .field("identifier", &self.id)
        .field("pld length", &self.pld.len())
        .finish()
    }
}

impl PartialEq for PacketFrame {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for PacketFrame {}