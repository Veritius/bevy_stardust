#[derive(Debug, Default)]
pub(super) struct PacketDataHeader(pub u8);

impl PacketDataHeader {
    /// Some irrelevant data is left out, reducing the overhead.
    pub const SLIMMED: u8 = 1 << 0;

    /// If this packet contains reliable data.
    pub const RELIABLE: u8 = 1 << 1;

    /// If this packet contains ordered data.
    pub const ORDERED: u8 = 1 << 2;

    /// If this packet contains multiple octet strings.
    pub const MULTIPLE: u8 = 1 << 3;

    /// If octet strings in the packet have different lengths.
    pub const LENGTHS: u8 = 1 << 4;

    #[inline(always)]
    pub fn is_slimmed(&self) -> bool {
        (self.0 & Self::SLIMMED) > 0
    }

    #[inline(always)]
    pub fn set_slimmed(&mut self) {
        self.0 |= Self::SLIMMED
    }

    #[inline(always)]
    pub fn is_reliable(&self) -> bool {
        (self.0 & Self::RELIABLE) > 0
    }

    #[inline(always)]
    pub fn set_reliable(&mut self) {
        self.0 |= Self::RELIABLE
    }

    #[inline(always)]
    pub fn is_ordered(&self) -> bool {
        (self.0 & Self::ORDERED) > 0
    }

    #[inline(always)]
    pub fn set_ordered(&mut self) {
        self.0 |= Self::ORDERED
    }

    #[inline(always)]
    pub fn is_multi_data(&self) -> bool {
        (self.0 & Self::MULTIPLE) > 0
    }

    #[inline(always)]
    pub fn set_multi_data(&mut self) {
        self.0 |= Self::MULTIPLE
    }

    #[inline(always)]
    pub fn is_dynamic_length(&self) -> bool {
        (self.0 & Self::LENGTHS) > 0
    }

    #[inline(always)]
    pub fn set_dynamic_length(&mut self) {
        self.0 |= Self::LENGTHS
    }
}