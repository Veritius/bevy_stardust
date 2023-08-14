use std::ops::{Add, Sub, AddAssign};
use bevy::reflect::{Reflect, TypePath};
use crate::shared::serialisation::{ManualBitSerialisation, BitWriter, BitReader, BitstreamError};

pub(super) const CHANNEL_ID_LIMIT: u32 = 2u32.pow(24);

pub trait Channel: TypePath + std::fmt::Debug + Send + Sync + 'static {}
impl<T: TypePath + std::fmt::Debug + Send + Sync + 'static> Channel for T {}

/// A unique 24-bit channel identifier. Internally a u32 for comparisons and performance. When sent over the network, it uses 3 bytes.
#[derive(Clone, Copy, Hash, Reflect, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChannelId(u32);

impl ChannelId {
    pub fn as_bytes(&self) -> [u8; 3] {
        let [_, a, b, c] = self.0.to_be_bytes();
        [a, b, c]
    }

    pub fn from_bytes(bytes: &[u8; 3]) -> Self {
        let [a, b, c] = *bytes;
        Self(u32::from_be_bytes([0, a, b, c]))
    }
}

impl std::fmt::Debug for ChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("c{}", self.0))
    }
}

impl ManualBitSerialisation for ChannelId {
    fn serialise(&self, writer: &mut impl BitWriter) {
        writer.write_bytes(self.as_bytes().iter().cloned());
    }

    fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
        let read = reader.read_bytes(3)?.into_boxed_slice();
        Ok(Self::from_bytes(&read[0..=3].try_into().unwrap()))
    }
}

impl TryFrom<u32> for ChannelId {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > CHANNEL_ID_LIMIT { return Err(()); }
        Ok(Self(value))
    }
}

impl Add<u32> for ChannelId {
    type Output = Result<Self, ()>;

    /// Adds `rhs` to the `ChannelId`'s value. Returns `Err(())` if the result would be greater than `2^24`.
    fn add(self, rhs: u32) -> Self::Output {
        let add = self.0.checked_add(rhs);
        if add.is_none() { return Err(()); }
        Self::try_from(add.unwrap())
    }
}

impl Sub<u32> for ChannelId {
    type Output = Result<Self, ()>;

    /// Subtracts `rhs` from the `ChannelId`'s value. Returns `Err(())` if the result would be greater than `2^24` or the operation would underflow.
    fn sub(self, rhs: u32) -> Self::Output {
        let sub = self.0.checked_sub(rhs);
        if sub.is_none() { return Err(()); }
        Self::try_from(sub.unwrap())
    }
}

impl AddAssign<u32> for ChannelId {
    /// Adds `rhs` to `self`. Panics if the result would be greater than `2^24`.
    fn add_assign(&mut self, rhs: u32) {
        *self = (*self + rhs)
            .expect("Tried to create a channel exceeding a value of 2^24")
    }
}