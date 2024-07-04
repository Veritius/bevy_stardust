//! The channel registry.

use std::{any::{type_name, TypeId}, collections::BTreeMap, ops::Deref, sync::Arc};
use bevy::{ecs::system::SystemParam, prelude::*};
use crate::prelude::ChannelConfiguration;
use super::{id::{Channel, ChannelId}, ToChannelId};

/// Access to registered channels and channel data.
/// 
/// This is only available after [`StardustPlugin`]`::`[`cleanup`] is called.
/// Attempts to access this type before this point will cause a panic.
/// 
/// For asynchronous contexts, [`clone_arc`](Self::clone_arc) can be used
/// to get a reference to the registry that will exist longer than the system.
/// This can be used in the [`ComputeTaskPool`] or [`AsyncComputeTaskPool`].
/// 
/// [`StardustPlugin`]: crate::plugin::StardustPlugin
/// [`cleanup`]: bevy::app::Plugin::cleanup
/// [`ComputeTaskPool`]: bevy::tasks::ComputeTaskPool
/// [`AsyncComputeTaskPool`]: bevy::tasks::AsyncComputeTaskPool
#[derive(SystemParam)]
pub struct Channels<'w> {
    // This hides the ChannelRegistryFinished type so that it
    // cannot be removed from the World, which would be bad
    finished: Res<'w, ChannelRegistryFinished>,
}

impl<'w> Channels<'w> {
    /// Returns an `Arc` to the underlying `ChannelRegistry`.
    /// This allows the registry to be used in asynchronous contexts.
    pub fn clone_arc(&self) -> Arc<ChannelRegistry> {
        self.finished.0.clone()
    }
}

impl<'a> AsRef<ChannelRegistry> for Channels<'a> {
    #[inline]
    fn as_ref(&self) -> &ChannelRegistry {
        &self.finished.0
    }
}

impl<'a> Deref for Channels<'a> {
    type Target = ChannelRegistry;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[derive(Resource)]
pub(super) struct ChannelRegistryBuilder(pub ChannelRegistry);

impl ChannelRegistryBuilder {
    pub fn finish(self) -> ChannelRegistryFinished {
        ChannelRegistryFinished(Arc::new(self.0))
    }
}

#[derive(Resource)]
pub(super) struct ChannelRegistryFinished(Arc<ChannelRegistry>);

impl Deref for ChannelRegistryFinished {
    type Target = ChannelRegistry;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// The inner registry 
pub struct ChannelRegistry {
    pub(super) channel_type_ids: BTreeMap<TypeId, ChannelId>,
    pub(super) channel_data: Vec<Registration>,
}

impl ChannelRegistry {
    pub(super) fn new() -> Self {
        Self {
            channel_type_ids: BTreeMap::new(),
            channel_data: vec![],
        }    
    }

    pub(super) fn register_channel<C: Channel>(
        &mut self,
        config: ChannelConfiguration,
    ) -> ChannelId {
        // Check we don't overrun the channel ID
        if self.channel_data.len() >= (u32::MAX as usize) {
            panic!("Exceeded channel limit of {}", u32::MAX);
        }
        
        // Check the channel doesn't already exist
        let type_id = TypeId::of::<C>();
        let type_name = type_name::<C>();
        if self.channel_type_ids.get(&type_id).is_some() {
            panic!("A channel was registered twice: {}", std::any::type_name::<C>());
        }

        // Add to map
        let channel_id = ChannelId::try_from(self.count()).unwrap();
        self.channel_type_ids.insert(type_id, channel_id);
        
        self.channel_data.push(Registration {
            metadata: ChannelMetadata {
                type_id,
                type_name,
                channel_id,
                _hidden: (),
            },

            config,
        });

        channel_id
    }

    /// Gets the id from the `ToChannelId` implementation.
    #[inline]
    pub fn id(&self, value: impl ToChannelId) -> Option<ChannelId> {
        value.to_channel_id(self)
    }

    pub(super) fn get_registration(&self, id: impl ToChannelId) -> Option<&Registration> {
        self.channel_data
            .get(Into::<usize>::into(id.to_channel_id(self)?))
    }

    /// Returns a reference to the channel metadata.
    pub fn metadata(&self, id: impl ToChannelId) -> Option<&ChannelMetadata> {
        self.get_registration(id).map(|v| &v.metadata)
    }

    /// Returns a reference to the channel configuration.
    pub fn config(&self, id: impl ToChannelId) -> Option<&ChannelConfiguration> {
        self.get_registration(id).map(|v| &v.config)
    }

    /// Returns whether the channel exists.
    pub fn exists(&self, id: ChannelId) -> bool {
        self.channel_data.len() >= Into::<usize>::into(id)
    }

    /// Returns how many channels currently exist.
    pub fn count(&self) -> u32 {
        TryInto::<u32>::try_into(self.channel_data.len()).unwrap()
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self {
            channel_type_ids: BTreeMap::new(),
            channel_data: vec![],
        }
    }
}

// AsRef is not reflexive, so we must implement it here
// https://doc.rust-lang.org/std/convert/trait.AsRef.html#reflexivity
impl AsRef<ChannelRegistry> for ChannelRegistry {
    #[inline]
    fn as_ref(&self) -> &ChannelRegistry { self }
}

/// Metadata about a channel, generated during channel registration.
pub struct ChannelMetadata {
    /// The channel's `TypeId`.
    pub type_id: TypeId,

    /// The channel's type name, from the `Any` trait.
    /// This is only useful for debugging, and is not stable across compilation.
    pub type_name: &'static str,

    /// The channel's sequential ID assigned by the registry.
    pub channel_id: ChannelId,

    _hidden: (),
}

pub(super) struct Registration {
    pub metadata: ChannelMetadata,
    pub config: ChannelConfiguration,
}