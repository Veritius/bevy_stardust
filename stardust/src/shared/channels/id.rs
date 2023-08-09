use bevy::reflect::{Reflect, TypePath};
use crate::shared::serialisation::{ManualBitSerialisation, BitWriter, BitReader, BitstreamError};

pub(super) const CHANNEL_ID_LIMIT: u32 = 2u32.pow(24);

pub trait Channel: TypePath + std::fmt::Debug + Send + Sync + 'static {}
impl<T: TypePath + std::fmt::Debug + Send + Sync + 'static> Channel for T {}

/// A unique 24-bit channel identifier.
#[derive(Clone, Copy, Hash, Reflect, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChannelId([u8;3]);

impl std::fmt::Debug for ChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ChannelId").field(&Into::<u32>::into(self.clone())).finish()
    }
}

impl ManualBitSerialisation for ChannelId {
    fn serialise(&self, writer: &mut impl BitWriter) {
        writer.write_bytes(self.0.iter().cloned());
    }

    fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
        Ok(Self(reader.read_bytes(3)?.try_into().unwrap()))
    }
}

impl TryFrom<&[u8]> for ChannelId {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 3 { return Err(()); }
        Ok(Self(value.try_into().unwrap()))
    }
}

impl TryFrom<u32> for ChannelId {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > CHANNEL_ID_LIMIT { return Err(())}
        let [_, a, b, c] = value.to_be_bytes();
        Ok(Self([a, b, c]))
    }
}

impl TryFrom<usize> for ChannelId {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        ChannelId::try_from(value as u32)
    }
}

impl From<ChannelId> for u32 {
    fn from(value: ChannelId) -> Self {
        let ChannelId([a, b, c]) = value;
        u32::from_be_bytes([0, a, b, c])
    }
}

impl From<ChannelId> for usize {
    fn from(value: ChannelId) -> Self {
        Into::<u32>::into(value) as usize
    }
}