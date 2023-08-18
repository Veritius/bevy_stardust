use std::ops::Deref;
use bevy::reflect::{Reflect, TypePath};
use crate::shared::integers::{u24, NIntegerError};

pub(super) const CHANNEL_ID_LIMIT: u32 = 2u32.pow(24);

pub trait Channel: TypePath + std::fmt::Debug + Send + Sync + 'static {}
impl<T: TypePath + std::fmt::Debug + Send + Sync + 'static> Channel for T {}

/// A unique 24-bit channel identifier.
#[derive(Debug, Clone, Copy, Hash, Reflect, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChannelId(u24);

impl Deref for ChannelId {
    type Target = u24;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<u32> for ChannelId {
    type Error = NIntegerError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match u24::try_from(value) {
            Ok(val) => Ok(Self(val)),
            Err(err) => Err(err),
        }
    }
}

impl From<[u8; 3]> for ChannelId {
    fn from(value: [u8; 3]) -> Self {
        Self(u24::from(value))
    }
}