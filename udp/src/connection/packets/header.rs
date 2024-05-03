use std::ops::{BitOr, BitOrAssign, BitAnd, BitAndAssign};

#[derive(Clone, Copy)]
pub(super) struct PacketHeaderFlags(pub u8);

impl PacketHeaderFlags {
    pub const EMPTY: Self = Self(0);

    pub const RELIABLE: Self = Self(1 << 0);

    #[inline]
    pub fn any_high(&self, mask: PacketHeaderFlags) -> bool {
        return (*self & mask).0 > 0;
    }
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