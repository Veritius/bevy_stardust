//! The channel registry.

use std::{any::TypeId, collections::BTreeMap, ops::{Deref, DerefMut}, sync::Arc};
use bevy::prelude::*;
use crate::prelude::ChannelConfiguration;
use super::{id::{Channel, ChannelId}, ToChannelId};

#[derive(Resource)]
pub(crate) struct ChannelRegistryMut(pub(crate) Box<ChannelRegistryInner>);

impl Deref for ChannelRegistryMut {
    type Target = ChannelRegistryInner;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChannelRegistryMut {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Read-only access to the channel registry, only available after app setup.
/// 
/// This can be freely and cheaply cloned, and will point to the same inner channel registry.
#[derive(Resource, Clone)]
pub struct ChannelRegistry(pub(crate) Arc<ChannelRegistryInner>);

impl Deref for ChannelRegistry {
    type Target = ChannelRegistryInner;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Stores channel configuration data. Accessible through the [`ChannelRegistry`] system parameter.
pub struct ChannelRegistryInner {
    pub(super) channel_type_ids: BTreeMap<TypeId, ChannelId>,
    pub(super) channel_data: Vec<ChannelData>,
}

impl ChannelRegistryInner {
    pub(in crate) fn new() -> Self {
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
        #[cfg(feature="reflect")]
        let type_path = C::type_path();
        if self.channel_type_ids.get(&type_id).is_some() {
            #[cfg(feature="reflect")]
            panic!("A channel was registered twice: {type_path}");
            #[cfg(not(feature="reflect"))]
            panic!("A channel was registered twice: {}", std::any::type_name::<C>());
        }

        // Add to map
        let channel_id = ChannelId::try_from(self.channel_count()).unwrap();
        self.channel_type_ids.insert(type_id, channel_id);
        
        #[cfg(feature="reflect")]
        self.channel_data.push(ChannelData {
            type_id,
            type_path,
            channel_id,

            config,
        });

        #[cfg(not(feature="reflect"))]
        self.channel_data.push(ChannelData {
            type_id,
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

impl Default for ChannelRegistryInner {
    fn default() -> Self {
        Self {
            channel_type_ids: BTreeMap::new(),
            channel_data: vec![],
        }
    }
}

/// Channel information generated when `add_channel` is run.
pub struct ChannelData {
    /// The channel's `TypeId`.
    pub type_id: TypeId,

    /// The channel's `TypePath` (from `bevy::reflect`)
    #[cfg(feature="reflect")]
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