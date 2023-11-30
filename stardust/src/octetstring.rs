//! Octet strings, arbitrary-sized sets of octets for transmission.

use std::{sync::Arc, ops::Deref};

/// An octet, aka byte. Smallest unit of communicable data Stardust can transfer.
pub type Octet = u8;

/// A string of octets (aka bytes).
/// 
/// This type can be cloned freely and cheaply, and will always point to the same space in memory.
/// Internally, it uses an `Arc` to count references, dropping the data when all copies are dropped.
#[derive(Debug, Clone)]
pub struct OctetString(Arc<[Octet]>);

impl OctetString {
    /// Returns a slice of the octet string.
    pub fn as_slice(&self) -> &[Octet] {
        &self.0
    }

    /// Returns how many octets are in this octet string.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<&[u8]> for OctetString {
    fn from(value: &[u8]) -> Self {
        let data = value.iter().cloned().collect::<Vec<Octet>>();
        Self(data.into())
    }
}

impl From<Vec<u8>> for OctetString {
    fn from(value: Vec<u8>) -> Self {
        Self(value.into())
    }
}

impl From<Box<[Octet]>> for OctetString {
    fn from(value: Box<[Octet]>) -> Self {
        Self(value.into())
    }
}

impl From<String> for OctetString {
    /// Writes the exact UTF-8 bytes of the `String` into an `OctetString`
    fn from(value: String) -> Self {
        value.into_bytes().into()
    }
}

impl Deref for OctetString {
    type Target = [Octet];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}