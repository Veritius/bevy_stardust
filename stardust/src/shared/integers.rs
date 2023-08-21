//! N-byte integers and network-efficient integers.
//! 
//! Types here are represented in memory as the smallest possible primitive, but are converted to bytes for network transport.
//! This means that a type like a `u24` is 4 bytes in memory, and 3 bytes over the wire.

use std::{fmt::Debug, ops::{Add, Sub}};
use bevy::reflect::Reflect;

#[allow(non_camel_case_types)]

/// 24-bit integer. 4 bytes in memory, 3 bytes in transport.
#[derive(Default, Clone, Copy, Reflect, PartialEq, Eq, PartialOrd, Ord)]
pub struct u24(u32);

impl u24 {
    pub const MIN: Self = Self(0);
    pub const MAX: Self = Self(2u32.pow(24));

    pub fn bytes(&self) -> [u8; 3] {
        let [_, a, b, c] = self.0.to_be_bytes();
        [a, b, c]
    }

    pub fn wrapping_add(self, rhs: Self) -> Self {
        let mut z = self.0.wrapping_add(rhs.0);
        let c = 2u32.pow(24);
        if z > c { z -= c + 1; }
        Self(z)
    }

    pub fn wrapping_sub(self, rhs: Self) -> Self {
        Self(self.0.checked_sub(rhs.0).unwrap_or_else(|| { todo!() }))
    }
}

impl Debug for u24 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::hash::Hash for u24 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.bytes().hash(state);
    }
}

impl Add for u24 {
    type Output = Result<Self, NIntegerError>;

    fn add(self, rhs: Self) -> Self::Output {
        let res = self.0.saturating_add(rhs.0);
        if res > 2u32.pow(24) { return Err(NIntegerError::OutOfRange) }
        Ok(Self(res))
    }
}

impl Sub for u24 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl From<[u8; 3]> for u24 {
    fn from(value: [u8; 3]) -> Self {
        Self(u32::from_be_bytes([0, value[0], value[1], value[2]]))
    }
}

impl From<u24> for [u8; 3] {
    fn from(value: u24) -> Self {
        value.bytes()
    }
}

impl From<u16> for u24 {
    fn from(value: u16) -> Self {
        Self(value as u32)
    }
}

impl TryFrom<u32> for u24 {
    type Error = NIntegerError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > 2u32.pow(24) { return Err(NIntegerError::OutOfRange) }
        Ok(Self(value))
    }
}

impl TryFrom<u64> for u24 {
    type Error = NIntegerError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value > 2u64.pow(24) { return Err(NIntegerError::OutOfRange) }
        Ok(Self(value as u32))
    }
}

impl TryFrom<u128> for u24 {
    type Error = NIntegerError;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        if value > 2u128.pow(24) { return Err(NIntegerError::OutOfRange) }
        Ok(Self(value as u32))
    }
}

impl TryFrom<usize> for u24 {
    type Error = NIntegerError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value > 2usize.pow(24) { return Err(NIntegerError::OutOfRange) }
        Ok(Self(value as u32))
    }
}

impl From<u24> for u32 {
    fn from(value: u24) -> Self {
        value.0
    }
}

impl From<u24> for u64 {
    fn from(value: u24) -> Self {
        value.0 as u64
    }
}

impl From<u24> for u128 {
    fn from(value: u24) -> Self {
        value.0 as u128
    }
}

impl From<u24> for usize {
    fn from(value: u24) -> Self {
        value.0 as usize
    }
}

/// Errors for dealing with n-byte integers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum NIntegerError {
    OutOfRange,
}

#[test]
fn u24_wrapping_test() {
    assert_eq!(u24::MAX.wrapping_add(1.into()), u24::MIN);
    assert_eq!(u24::from(16u16).wrapping_add(1u16.into()), u24::from(17u16));
    // assert_eq!(u24::MIN.wrapping_sub(1.into()), u24::MAX);
    assert_eq!(u24::from(16u16).wrapping_sub(1u16.into()), u24::from(15u16));
}