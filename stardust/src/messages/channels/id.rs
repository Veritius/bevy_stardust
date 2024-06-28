use std::any::Any;
use bevy::prelude::*;
use super::ChannelRegistry;

/// Types that can be used to identify channels within the type system.
/// Once registered to the `App`, this type has a [`ChannelId`] assigned to it.
pub trait Channel: Any {}

impl<T: Any> Channel for T {}

/// A unique identifier for a channel, generated during application setup.
/// 
/// A `ChannelId` is used to identify a channel without type information,
/// such as in a transport layer or associative arrays where `TypeId`
/// would be excessive. Channel registration also ensures that the same
/// `ChannelId` refers to the same channel, regardless of compilation.
/// This only holds true if the [ordering constraints](super) are obeyed.
/// 
/// Note that channel identifiers are only unique to the
/// `World` belonging to the `App` they were registered to.
/// Using them in a different `World` or `App` may panic,
/// or have additional consequences because of transport
/// layers, such as causing undefined behavior.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Reflect)]
#[repr(transparent)]
pub struct ChannelId(u32);

impl std::fmt::Debug for ChannelId {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<u32> for ChannelId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<[u8;4]> for ChannelId {
    fn from(value: [u8;4]) -> Self {
        Self(u32::from_be_bytes(value))
    }
}

impl From<ChannelId> for u32 {
    fn from(value: ChannelId) -> Self {
        value.0
    }
}

impl From<ChannelId> for usize {
    fn from(value: ChannelId) -> Self {
        value.0 as usize
    }
}

impl From<ChannelId> for [u8;4] {
    fn from(value: ChannelId) -> Self {
        value.0.to_be_bytes()
    }
}

/// Types that can be used to access channel data in a channel registry.
pub trait ToChannelId {
    /// Convert the type to a `ChannelId`.
    fn to_channel_id(&self, registry: impl AsRef<ChannelRegistry>) -> Option<ChannelId>;
}

impl ToChannelId for ChannelId {
    #[inline]
    fn to_channel_id(&self, _: impl AsRef<ChannelRegistry>) -> Option<ChannelId> {
        Some(self.clone())
    }
}

impl ToChannelId for std::any::TypeId {
    #[inline]
    fn to_channel_id(&self, registry: impl AsRef<ChannelRegistry>) -> Option<ChannelId> {
        registry.as_ref().channel_type_ids.get(&self).cloned()
    }
}