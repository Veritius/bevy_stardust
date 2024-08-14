//! Unique network identifier values for sharing across connections.

/// A unique network identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct NetId(u64);

impl NetId {
    /// Creates a new `NetId`.
    /// 
    /// The value of the highest bit is always overwritten.
    /// If this is not desirable behavior, use [`new_checked`](Self::new_checked).
    pub fn new(side: Side, mut index: u64) -> Self {
        // Clear the first bit
        index &= u64::MAX >> 1;

        // Set the flag high for right
        if side == Side::Right {
            index |= 1u64 << 63;
        };

        // We're done, return
        return Self(index);
    }

    /// Creates a new `NetId`, checking if `value` is out of range.
    /// 
    /// If you don't need to check, use [`new`](Self::new).
    pub fn new_checked(side: Side, index: u64) -> Result<Self, ()> {
        if index & 1u64 >> 63 != 0 { return Err(()) }
        return Ok(Self::new(side, index));
    }

    /// Create a [`NetId`] from its bit representation.
    #[inline]
    pub fn from_bits(bits: u64) -> Self {
        Self(bits.to_be())
    }

    /// Returns the bit representation of the [`NetId`]
    #[inline]
    pub fn into_bits(self) -> u64 {
        self.0.to_be()
    }

    /// Returns the [`Side`] that created this ID.
    pub fn side(&self) -> Side {
        match self.0 >> 63 == 0 {
            true => Side::Left,
            false => Side::Right,
        }
    }

    /// Returns the index of the identifier.
    pub fn index(&self) -> u64 {
        self.0 & u64::MAX >> 1
    }
}

/// The side of a connection, used to ensure [`NetId`] values are unique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum Side {
    Left,
    Right,
}

#[test]
fn ident_bits_test() {
    fn test_id(value: NetId, side: Side, index: u64) {
        assert_eq!(value.side(), side);
        assert_eq!(value.index(), index);
    }

    test_id(NetId::new(Side::Left, 0), Side::Left, 0);
    test_id(NetId::new(Side::Right, 0), Side::Right, 0);
    test_id(NetId::new(Side::Left, 1), Side::Left, 1);
    test_id(NetId::new(Side::Right, 1), Side::Right, 1);
    test_id(NetId::new(Side::Left, 5), Side::Left, 5);
    test_id(NetId::new(Side::Right, 5), Side::Right, 5);
    test_id(NetId::new(Side::Left, 25), Side::Left, 25);
    test_id(NetId::new(Side::Right, 25), Side::Right, 25);
    test_id(NetId::new(Side::Left, 543), Side::Left, 543);
    test_id(NetId::new(Side::Right, 543), Side::Right, 543);
    test_id(NetId::new(Side::Left, 23553), Side::Left, 23553);
    test_id(NetId::new(Side::Right, 23553), Side::Right, 23553);
}