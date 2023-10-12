//! Packet sequencing and bitfields.

use std::{ops::{Add, Sub}, cmp::Ordering, fmt::Display};

/// A number that can be used to store the sequence value of a packet.
pub trait SequenceNumber: Sized + Clone + Copy + Display + PartialOrd + Add<Output = Self> + Sub<Output = Self> {
    const BYTE_SIZE: u8;
    const BIT_SIZE: u8 = Self::BYTE_SIZE * 8;
    const VAL_MIN: Self;
    const VAL_MID: Self;
    const VAL_MAX: Self;

    /// Try to create `Self` from a slice of bytes.
    fn from_bytes(bytes: &[u8]) -> Option<Self>;

    // TODO: Don't use allocation, use a fixed size array with size defined by the trait implementor
    // Not sure how to do that at the moment and my Internet connection isn't working.
    fn to_bytes(&self) -> Box<[u8]>;

    fn wrapping_add(&self, other: Self) -> Self {
        todo!()
    }

    fn wrapping_sub(&self, other: Self) -> Self {
        todo!()
    }

    /// Compare sequence numbers, taking wrapping into consideration.
    fn wrapping_compare(&self, other: Self) -> Ordering {
        if *self == other { return Ordering::Equal }
        match
            ( ( *self > other ) && ( *self - other <= Self::VAL_MID ) ) ||
            ( ( *self < other ) && ( other - *self > Self::VAL_MID ) )
        {
            true => Ordering::Greater,
            false => Ordering::Less,
        }
    }
}

impl SequenceNumber for u8 {
    const BYTE_SIZE: u8 = 1;
    const VAL_MIN: Self = u8::MIN;
    const VAL_MID: Self = 127;
    const VAL_MAX: Self = u8::MAX;

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() == 0 { return None }
        Some(bytes[0])
    }

    fn to_bytes(&self) -> Box<[u8]> {
        Box::new(self.to_be_bytes())
    }
}

impl SequenceNumber for u16 {
    const BYTE_SIZE: u8 = 2;
    const VAL_MIN: Self = u16::MIN;
    const VAL_MID: Self = 32768;
    const VAL_MAX: Self = u16::MAX;

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 2 { return None }
        Some(u16::from_be_bytes(bytes[0..1].try_into().unwrap()))
    }

    fn to_bytes(&self) -> Box<[u8]> {
        Box::new(self.to_be_bytes())
    }
}

impl SequenceNumber for u32 {
    const BYTE_SIZE: u8 = 4;
    const VAL_MIN: Self = u32::MIN;
    const VAL_MID: Self = 2147483648;
    const VAL_MAX: Self = u32::MAX;

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 4 { return None }
        Some(u32::from_be_bytes(bytes[0..3].try_into().unwrap()))
    }

    fn to_bytes(&self) -> Box<[u8]> {
        Box::new(self.to_be_bytes())
    }
}

/// A value that can store the received status of the past few packets.
pub trait SequenceBitset: Sized {
    const BYTE_SIZE: u8;
    const BIT_SIZE: u8 = Self::BYTE_SIZE * 8;

    fn set_bit(&mut self, idx: u8);
}

impl SequenceBitset for u32 {
    const BYTE_SIZE: u8 = 4;

    #[inline]
    fn set_bit(&mut self, idx: u8) {
        todo!()
    }
}

impl SequenceBitset for u64 {
    const BYTE_SIZE: u8 = 8;

    #[inline]
    fn set_bit(&mut self, idx: u8) {
        todo!()
    }
}

impl SequenceBitset for u128 {
    const BYTE_SIZE: u8 = 16;

    #[inline]
    fn set_bit(&mut self, idx: u8) {
        todo!()
    }
}

#[test]
fn wrapping_compare_test() {
    // Simple comparisons
    assert_eq!(0u16.wrapping_compare(0), Ordering::Equal);
    assert_eq!(1u16.wrapping_compare(0), Ordering::Greater);
    assert_eq!(0u16.wrapping_compare(1), Ordering::Less);

    // Wrapping numbers
    assert_eq!(60000u16.wrapping_compare(5), Ordering::Less);
    assert_eq!(5u16.wrapping_compare(60000), Ordering::Greater);
}