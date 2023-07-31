pub type Octet = u8;

/// A string of octets (aka bytes).
pub struct OctetString(Box<[Octet]>);

impl OctetString {
    pub fn as_slice(&self) -> &[Octet] {
        &self.0
    }
}

impl From<Vec<u8>> for OctetString {
    fn from(value: Vec<u8>) -> Self {
        Self(value.into_boxed_slice())
    }
}

impl From<Box<[Octet]>> for OctetString {
    fn from(value: Box<[Octet]>) -> Self {
        Self(value)
    }
}