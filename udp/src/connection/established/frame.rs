use std::{fmt::Debug, ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign}};
use bevy_stardust::prelude::*;
use bytes::Bytes;
use crate::sequences::SequenceId;

#[derive(Debug)]
pub(super) struct Frame {
    pub flags: FrameFlags,
    pub ident: u32,
    pub order: Option<SequenceId>,
    pub bytes: Bytes,
}

impl Frame {
    pub fn transport_message(
        flags: FrameFlags,
        order: Option<SequenceId>,
        bytes: Bytes,
    ) -> Self {
        Self {
            flags,
            ident: 0,
            order,
            bytes,
        }
    }

    pub fn stardust_message(
        channel: ChannelId,
        data: &ChannelData,
        order: Option<SequenceId>,
        bytes: Bytes,
    ) -> Self {
        let mut flags = FrameFlags::default();

        if data.reliable == ReliabilityGuarantee::Reliable {
            flags |= FrameFlags::RELIABLE;
        }

        Self {
            flags,
            ident: u32::from(channel).wrapping_add(1),
            order,
            bytes,
        }
    }
}

pub(super) struct FrameFlags(pub u32);

impl FrameFlags {
    pub const RELIABLE: Self = Self(1 << 0);
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