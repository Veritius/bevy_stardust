/// A value that may or may not identify a packet. Avoids making a value of `0`.
/// 
/// Values of `1..=65535` identify packets, while a value of `0` means 'no packet'.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SequenceValue(pub u16);

impl SequenceValue {
    pub const NON: Self = Self(0);
    pub const MIN: Self = Self(1);
    pub const MID: Self = Self(32768);
    pub const MAX: Self = Self(u16::MAX);

    #[inline(always)]
    pub fn next(self) -> Self {
        self.wrapping_add(1)
    }

    #[inline(always)]
    pub fn previous(self) -> Self {
        self.wrapping_sub(1)
    }

    #[inline]
    pub fn wrapping_add(&self, rhs: u16) -> Self {
        let mut v = self.0.wrapping_add(rhs);
        if v == 0 { v = 1; }
        Self(v)
    }

    #[inline]
    pub fn wrapping_sub(&self, rhs: u16) -> Self {
        let mut v = self.0.wrapping_sub(rhs);
        if v == 0 { v = u16::MAX; }
        Self(v)
    }
}

impl Default for SequenceValue {
    fn default() -> Self {
        Self::MIN
    }
}

impl From<[u8; 2]> for SequenceValue {
    fn from(value: [u8; 2]) -> Self {
        Self(u16::from_be_bytes(value))
    }
}

impl From<u16> for SequenceValue {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<SequenceValue> for [u8; 2] {
    fn from(value: SequenceValue) -> Self {
        value.0.to_be_bytes()
    }
}

impl From<SequenceValue> for u16 {
    fn from(value: SequenceValue) -> Self {
        value.0
    }
}