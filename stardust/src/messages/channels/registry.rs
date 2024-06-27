//! The channel registry.

use std::{any::TypeId, collections::BTreeMap, ops::Deref, sync::Arc};
use bevy::{ecs::system::SystemParam, prelude::*};
use crate::prelude::ChannelConfiguration;
use super::{id::{Channel, ChannelId}, ToChannelId};

/// Access to registered channels and channel data.
/// 
/// This is only available after [`StardustPlugin`]`::`[`finish`] is called.
/// Attempts to call before this point will cause a panic.
/// 
/// For asynchronous contexts, [`clone_arc`](Self::clone_arc) can be used
/// to get a reference to the registry that will exist longer than the system.
/// This can be used in the [`ComputeTaskPool`] or [`AsyncComputeTaskPool`].
/// 
/// [`StardustPlugin`]: crate::plugin::StardustPlugin
/// [`finish`]: bevy::app::Plugin::finish
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

/// The inner registry 
pub struct ChannelRegistry {
    pub(super) channel_type_ids: BTreeMap<TypeId, ChannelId>,
    pub(super) channel_data: Vec<ChannelData>,
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
        let type_path = C::type_path();
        if self.channel_type_ids.get(&type_id).is_some() {
            panic!("A channel was registered twice: {}", std::any::type_name::<C>());
        }

        // Add to map
        let channel_id = ChannelId::try_from(self.channel_count()).unwrap();
        self.channel_type_ids.insert(type_id, channel_id);
        
        self.channel_data.push(ChannelData {
            type_id,
            type_path,
            channel_id,

            config,
        });

        channel_id
    }

    /// Gets the id from the `ToChannelId` implementation.
    #[inline]
    pub fn channel_id(&self, value: impl ToChannelId) -> Option<ChannelId> {
        value.to_channel_id(self)
    }

    /// Returns a reference to the channel configuration.
    pub fn channel_config(&self, id: impl ToChannelId) -> Option<&ChannelData> {
        self.channel_data.get(Into::<usize>::into(id.to_channel_id(self)?))
    }

    /// Returns whether the channel exists.
    pub fn channel_exists(&self, id: ChannelId) -> bool {
        self.channel_data.len() >= Into::<usize>::into(id)
    }

    /// Returns how many channels currently exist.
    pub fn channel_count(&self) -> u32 {
        TryInto::<u32>::try_into(self.channel_data.len()).unwrap()
    }

    /// Returns an iterator of all existing channel ids.
    pub fn channel_ids(&self) -> impl Iterator<Item = ChannelId> {
        (0..self.channel_count()).into_iter()
        .map(|f| ChannelId::try_from(f).unwrap())
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

/// Channel information generated when `add_channel` is run.
pub struct ChannelData {
    /// The channel's `TypeId`.
    pub type_id: TypeId,

    /// The channel's `TypePath` (from `bevy::reflect`)
    pub type_path: &'static str,

    /// The channel's sequential ID assigned by the registry.
    pub channel_id: ChannelId,

    /// The config of the channel.
    /// Since `ChannelData` implements `Deref` for `ChannelConfiguration`, this is just clutter.
    config: ChannelConfiguration,
}

impl std::ops::Deref for ChannelData {
    type Target = ChannelConfiguration;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}