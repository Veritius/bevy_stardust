//! Packet sequencing and bitfields.

use std::{ops::Add, cmp::Ordering};

/// A number that can be used to store the sequence value of a packet.
pub trait SequenceNumber: Sized + PartialOrd + Add {
    const BYTE_SIZE: u8;
    const BIT_SIZE: u8 = Self::BYTE_SIZE * 8;

    fn from_bytes(bytes: &[u8]) -> Option<Self>;
    // TODO: Don't use allocation, use a fixed size array with size defined by the trait implementor
    // Not sure how to do that at the moment and my Internet connection isn't working.
    fn to_bytes(&self) -> Box<[u8]>;
    fn wrapping_compare(&self, other: Self) -> Ordering;
}

impl SequenceNumber for u8 {
    const BYTE_SIZE: u8 = 1;

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() == 0 { return None }
        Some(bytes[0])
    }

    fn to_bytes(&self) -> Box<[u8]> {
        Box::new(self.to_be_bytes())
    }
    
    fn wrapping_compare(&self, other: Self) -> Ordering {
        todo!()
    }
}

impl SequenceNumber for u16 {
    const BYTE_SIZE: u8 = 2;

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 2 { return None }
        Some(u16::from_be_bytes(bytes[0..1].try_into().unwrap()))
    }

    fn to_bytes(&self) -> Box<[u8]> {
        Box::new(self.to_be_bytes())
    }

    fn wrapping_compare(&self, other: Self) -> Ordering {
        todo!()
    }
}

impl SequenceNumber for u32 {
    const BYTE_SIZE: u8 = 4;

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 4 { return None }
        Some(u32::from_be_bytes(bytes[0..3].try_into().unwrap()))
    }

    fn to_bytes(&self) -> Box<[u8]> {
        Box::new(self.to_be_bytes())
    }

    fn wrapping_compare(&self, other: Self) -> Ordering {
        todo!()
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