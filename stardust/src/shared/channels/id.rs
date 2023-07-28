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

impl TryFrom<&[u8]> for ChannelId {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 3 { return Err(()); }
        Ok(Self(value.try_into().unwrap()))
    }
}

impl From<u32> for ChannelId {
    fn from(value: u32) -> Self {
        if value > CHANNEL_ID_LIMIT { panic!("Can't create a channel ID with a value") }
        let [_, a, b, c] = value.to_be_bytes();
        Self([a, b, c])
    }
}

impl From<usize> for ChannelId {
    fn from(value: usize) -> Self {
        ChannelId::from(value as u32)
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