//! Types that can be used to interface with Stardust's message reading and writing APIs.
//!
//! Note: In the following examples, `#[derive(Reflect)]` is only needed with the `reflect` feature flag.
//! 
//! ```ignore
//! // Defining a channel type is simple
//! #[derive(Reflect)]
//! pub struct MyChannel;
//! 
//! // You can make channels private
//! #[derive(Reflect)]
//! struct MyPrivateChannel;
//! 
//! // You can make channels with generic type bounds too
//! #[derive(Reflect)]
//! struct MyGenericChannel<T: Channel>(PhantomData<T>);
//! ```
//! 
//! In Stardust, `Channel` trait objects are just used for their type data.
//! The type itself isn't actually stored. That means you can do things like this.
//! 
//! ```ignore
//! #[derive(Reflect, Event)]
//! pub struct MovementEvent(pub Vec3);
//!
//! fn main() {
//!     let mut app = App::new();
//! 
//!     app.add_plugins((DefaultPlugins, StardustPlugin));
//! 
//!     app.add_event::<MovementEvent>();
//!     app.add_channel::<MovementEvent>(ChannelConfiguration {
//!         reliable: ReliabilityGuarantee::Unreliable,
//!         ordered: OrderingGuarantee::Unordered,
//!         fragmented: false,
//!         priority: 0,
//!     });
//! }
//! ```

use std::{marker::PhantomData, ops::Deref};
use bevy::prelude::*;
use super::ChannelRegistryInner;

/// Marker trait for channels. See the [module level documentation](self) for more information.
#[cfg(not(feature="reflect"))]
pub trait Channel: Send + Sync + 'static {}

#[cfg(not(feature="reflect"))]
impl<T: Send + Sync + 'static> Channel for T {}

#[cfg(feature="reflect")]
use bevy::reflect::*;

/// Marker trait for channels. See the [module level documentation](self) for more information.
#[cfg(feature="reflect")]
pub trait Channel: Reflect + TypePath + GetTypeRegistration + Send + Sync + 'static {}

#[cfg(feature="reflect")]
impl<T: Reflect + TypePath + GetTypeRegistration + Send + Sync + 'static> Channel for T {}

/// Typed marker component for filtering channel entities.
#[derive(Component)]
pub(super) struct ChannelMarker<C: Channel>(pub PhantomData<C>);

impl<C: Channel> Default for ChannelMarker<C> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// A sequential channel identifier that can be used to access data without type information.
/// 
/// Channel identifiers are generated by the `ChannelRegistry` and are unique to the `World` they originated from.
/// Attempting to use a `ChannelId` in another `World` will probably panic, or give you unintended results.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Reflect)]
#[repr(transparent)]
pub struct ChannelId(u32);

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
pub trait ToChannelId: sealed::Sealed {
    /// Convert the type to a `ChannelId`
    fn to_channel_id(&self, registry: impl Deref<Target = ChannelRegistryInner>) -> Option<ChannelId>;
}

impl ToChannelId for ChannelId {
    #[inline]
    fn to_channel_id(&self, _: impl Deref<Target = ChannelRegistryInner>) -> Option<ChannelId> {
        Some(self.clone())
    }
}

impl ToChannelId for std::any::TypeId {
    fn to_channel_id(&self, registry: impl Deref<Target = ChannelRegistryInner>) -> Option<ChannelId> {
        registry.channel_type_ids.get(&self).cloned()
    }
}

#[cfg(feature="reflect")]
impl ToChannelId for &dyn bevy::reflect::Reflect {
    fn to_channel_id(&self, registry: impl Deref<Target = ChannelRegistryInner>) -> Option<ChannelId> {
        self.type_id().to_channel_id(registry)
    }
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::ChannelId {}
    impl Sealed for std::any::TypeId {}

    #[cfg(feature="reflect")]
    impl Sealed for &dyn bevy::reflect::Reflect {}
}