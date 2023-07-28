use bevy::reflect::Reflect;
use crate::shared::serialisation::{ManualBitSerialisation, BitWriter, BitReader, BitstreamError};

pub(super) const CHANNEL_ID_LIMIT: u32 = 2u32.pow(24);

pub trait Channel: std::fmt::Debug + Send + Sync + 'static {}
impl<T: std::fmt::Debug + Send + Sync + 'static> Channel for T {}

/// A unique 24-bit channel identifier.
#[derive(Debug, Default, Clone, Copy, Hash, Reflect, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChannelId([u8;3]);

impl ManualBitSerialisation for ChannelId {
    fn serialise(&self, writer: &mut impl BitWriter) {
        writer.write_bytes(self.0.iter().cloned());
    }

    fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
        Ok(Self(reader.read_bytes(3)?.try_into().unwrap()))
    }
}

impl From<u32> for ChannelId {
    fn from(value: u32) -> Self {
        todo!()
    }
}