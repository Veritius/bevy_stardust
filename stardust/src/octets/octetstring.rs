//! Octet strings, arbitrary-sized sets of octets for transmission.

use std::sync::Arc;

pub type Octet = u8;

/// A string of octets (aka bytes).
/// 
/// This type can be cloned freely and cheaply, and will always point to the same space in memory.
/// Internally, it uses an `Arc` to count references, dropping the data when all copies are dropped.
#[derive(Debug, Clone)]
pub struct OctetString(Arc<Vec<Octet>>);

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
        Self(Arc::new(data))
    }
}

impl From<Vec<u8>> for OctetString {
    fn from(value: Vec<u8>) -> Self {
        Self(Arc::new(value))
    }
}

impl From<Box<[Octet]>> for OctetString {
    fn from(value: Box<[Octet]>) -> Self {
        Self(Arc::new(value.into_vec()))
    }
}

impl From<String> for OctetString {
    fn from(value: String) -> Self {
        value.into_bytes().into()
    }
}