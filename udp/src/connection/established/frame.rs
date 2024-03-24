use std::{fmt::Debug, ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign}};
use bytes::Bytes;

#[derive(Debug)]
pub(super) struct Frame {
    pub flags: FrameFlags,
    pub ident: u32,
    pub bytes: Bytes,
}

#[derive(Clone, Copy)]
pub(super) struct FrameFlags(pub u32);

impl FrameFlags {
    pub const RELIABLE: Self = Self(1 << 0);
    pub const ORDERED: Self = Self(1 << 1);
}

impl Default for FrameFlags {
    #[inline]
    fn default() -> Self {
        Self(0)
    }
}

impl Debug for FrameFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:b}", self.0))
    }
}

impl BitOr for FrameFlags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for FrameFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl BitAnd for FrameFlags {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for FrameFlags {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}