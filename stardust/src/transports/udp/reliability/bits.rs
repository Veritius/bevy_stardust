//! Packet sequencing and bitfields.

use std::{cmp::Ordering, fmt::Display};

// A pretty bad way to avoid having to allocate, but it's good enough.
pub struct SemiByteArray(pub usize, pub [u8; 16]);

impl SemiByteArray {
    pub fn new() -> Self {
        Self(0, [0; 16])
    }

    pub fn read(&self) -> &[u8] {
        &self.1[0..self.0]
    }
}

/// A number that can be used to store the sequence value of a packet.
pub trait SequenceNumber: Sized + Clone + Copy + Display {
    const SQ_BYTE_SIZE: u8;
    const BIT_SIZE: u8 = Self::SQ_BYTE_SIZE * 8;
    const VAL_ONE: Self;
    const VAL_MIN: Self;
    const VAL_MAX: Self;

    /// Try to create `Self` from a slice of bytes.
    fn from_bytes(bytes: &[u8]) -> Option<Self>;

    fn to_bytes(&self) -> SemiByteArray;

    fn wrapping_add(self, other: Self) -> Self;

    fn wrapping_sub(self, other: Self) -> Self;

    /// Compare sequence numbers, taking wrapping into consideration.
    fn wrapping_compare(self, other: Self) -> Ordering;

    fn wrapping_difference(&self, other: Self) -> Self;

    /// Convert to u8, saturating at u8::MAX if too large.
    fn to_u8(self) -> u8;
}

macro_rules! impl_sequence_number_primitive {
    ($i:ident, $h:expr) => {

        impl SequenceNumber for $i {
            const SQ_BYTE_SIZE: u8 = std::mem::size_of::<Self>() as u8;
            const VAL_ONE: Self = 1;
            const VAL_MIN: Self = $i::MIN;
            const VAL_MAX: Self = $i::MAX;

            fn from_bytes(bytes: &[u8]) -> Option<Self> {
                if bytes.len() < (((Self::SQ_BYTE_SIZE as Self + Self::VAL_ONE))).try_into().unwrap() { return None }
                Some(u8::from_be_bytes(bytes[0..(Self::SQ_BYTE_SIZE as usize - 1)].try_into().unwrap()).into())
            }

            #[inline]
            fn to_bytes(&self) -> SemiByteArray {
                let mut sbi = SemiByteArray::new();
                for (i, x) in self.to_be_bytes().iter().enumerate() {
                    sbi.0 = i;
                    sbi.1[i] = *x;
                }
                sbi
            }

            #[inline]
            fn wrapping_add(self, other: Self) -> Self {
                $i::wrapping_add(self, other)
            }

            #[inline]
            fn wrapping_sub(self, other: Self) -> Self {
                $i::wrapping_sub(self, other)
            }

            fn wrapping_compare(self, other: Self) -> Ordering {
                gaffer_wrapping_compare!(self, other, $h)
            }

            #[inline]
            fn wrapping_difference(&self, other: Self) -> Self {
                match self.wrapping_compare(other) {
                    Ordering::Greater => self.wrapping_sub(other),
                    Ordering::Equal => 0,
                    Ordering::Less => other.wrapping_sub(*self),
                }
            }

            #[inline]
            fn to_u8(self) -> u8 {
                self.min(u8::MAX as $i) as u8
            }
        }
    };
}

// Wrapping comparison based on Gaffer on Games' algorithm.
macro_rules! gaffer_wrapping_compare {
    ($s:ident, $o:ident, $e:expr) => {
        if $s == $o { 
            return Ordering::Equal
        } else {
            return match
                ( ( $s > $o ) && ( $s - $o <= $e ) ) ||
                ( ( $s < $o ) && ( $o - $s > $e ) )
            {
                true => Ordering::Greater,
                false => Ordering::Less,
            }
        }
    };
}

impl_sequence_number_primitive!(u8, 127);
impl_sequence_number_primitive!(u16, 32768);
impl_sequence_number_primitive!(u32, 2147483648);

/// A value that can store the received status of the past few packets.
pub trait SequenceBitset: Sized {
    const BYTE_SIZE: u8;
    const BIT_SIZE: u8 = Self::BYTE_SIZE * 8;

    fn new() -> Self;
    fn shift_left(&mut self, idx: u8);
    fn set_bit_on(&mut self, idx: u8);
    fn to_bytes(&self) -> SemiByteArray;
}

impl SequenceBitset for u32 {
    const BYTE_SIZE: u8 = 4;

    fn new() -> Self { 0 }

    #[inline]
    fn shift_left(&mut self, idx: u8) {
        *self <<= idx
    }

    #[inline]
    fn set_bit_on(&mut self, idx: u8) {
        let mut mask = 1u32;
        mask <<= idx;
        *self |= mask;
    }

    #[inline]
    fn to_bytes(&self) -> SemiByteArray {
        let mut sbi = SemiByteArray::new();
        for (i, x) in self.to_be_bytes().iter().enumerate() {
            sbi.0 = i;
            sbi.1[i] = *x;
        }
        sbi
    }
}

impl SequenceBitset for u64 {
    const BYTE_SIZE: u8 = 8;

    fn new() -> Self { 0 }

    #[inline]
    fn shift_left(&mut self, idx: u8) {
        *self <<= idx
    }

    #[inline]
    fn set_bit_on(&mut self, idx: u8) {
        let mut mask = 1u64;
        mask <<= idx;
        *self |= mask;
    }

    #[inline]
    fn to_bytes(&self) -> SemiByteArray {
        let mut sbi = SemiByteArray::new();
        for (i, x) in self.to_be_bytes().iter().enumerate() {
            sbi.0 = i;
            sbi.1[i] = *x;
        }
        sbi
    }
}

impl SequenceBitset for u128 {
    const BYTE_SIZE: u8 = 16;

    fn new() -> Self { 0 }

    #[inline]
    fn shift_left(&mut self, idx: u8) {
        *self <<= idx
    }

    #[inline]
    fn set_bit_on(&mut self, idx: u8) {
        let mut mask = 1u128;
        mask <<= idx;
        *self |= mask;
    }

    #[inline]
    fn to_bytes(&self) -> SemiByteArray {
        let mut sbi = SemiByteArray::new();
        for (i, x) in self.to_be_bytes().iter().enumerate() {
            sbi.0 = i;
            sbi.1[i] = *x;
        }
        sbi
    }
}

#[test]
fn sequence_numbers() {
    // Simple comparisons
    assert_eq!(0u16.wrapping_compare(0), Ordering::Equal);
    assert_eq!(1u16.wrapping_compare(0), Ordering::Greater);
    assert_eq!(0u16.wrapping_compare(1), Ordering::Less);

    // Wrapping numbers
    assert_eq!(60000u16.wrapping_compare(5), Ordering::Less);
    assert_eq!(5u16.wrapping_compare(60000), Ordering::Greater);
    
    // Wrapping differences
    assert_eq!(600u16.wrapping_difference(605), 5);
    assert_eq!(100u16.wrapping_difference(100), 0);
    assert_eq!(u16::MAX.wrapping_difference(u16::MIN), 1);
}