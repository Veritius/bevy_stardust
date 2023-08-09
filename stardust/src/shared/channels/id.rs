use bevy::reflect::{Reflect, TypePath};
use crate::shared::serialisation::{ManualBitSerialisation, BitWriter, BitReader, BitstreamError};

pub(super) const CHANNEL_ID_LIMIT: u32 = 2u32.pow(24);

pub trait Channel: TypePath + std::fmt::Debug + Send + Sync + 'static {}
impl<T: TypePath + std::fmt::Debug + Send + Sync + 'static> Channel for T {}

/// A unique 24-bit channel identifier. Internally a u32 for comparisons and performance.
#[derive(Debug, Clone, Copy, Hash, Reflect, PartialEq, Eq, PartialOrd, Ord)]
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