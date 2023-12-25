//! The channel registry.

use std::{collections::BTreeMap, any::TypeId};
use bevy::prelude::*;
use crate::prelude::ChannelConfiguration;
use super::id::{Channel, ChannelId};

/// Stores information related to type ids.
#[derive(Resource)]
pub struct ChannelRegistry {
    channel_count: u32,
    channel_type_ids: BTreeMap<TypeId, ChannelId>,
    channel_data: Vec<ChannelData>,
}

impl ChannelRegistry {
    pub(in crate) fn new() -> Self {
        Self {
            channel_count: 0,
            channel_type_ids: BTreeMap::new(),
            channel_data: vec![],
        }    
    }

    pub(super) fn register_channel<C: Channel>(
        &mut self,
        config: ChannelConfiguration,
        entity: Entity,
    ) -> ChannelId {
        // Check we don't overrun the channel ID
        if self.channel_count == u32::MAX {
            panic!("Exceeded channel limit of {}", u32::MAX);
        }
        
        // Check the channel doesn't already exist
        let type_id = TypeId::of::<C>();
        let type_path = C::type_path();
        if self.channel_type_ids.get(&type_id).is_some() {
            panic!("A channel was registered twice: {type_path}");
        }

        // Add to map
        let channel_id = ChannelId::try_from(self.channel_count).unwrap();
        self.channel_type_ids.insert(type_id, channel_id);
        self.channel_data.push(ChannelData {
            type_id,
            type_path,
            channel_id,

            entity_id: entity,
            config,
        });
        self.channel_count += 1;

        channel_id
    }

    /// Returns the channel ID if `reflect` is a registered type
    pub fn get_from_reflect(&self, reflect: &dyn Reflect) -> Option<(ChannelId, &ChannelData)> {
        self.get_from_type_inner(reflect.as_any().type_id())
    }

    /// Returns the ChannelId and a reference to the ChannelConfig if `C` is a registered type
    pub fn get_from_type<C: Channel>(&self) -> Option<(ChannelId, &ChannelData)> {
        self.get_from_type_inner(TypeId::of::<C>())
    }

    fn get_from_type_inner(&self, typeid: TypeId) -> Option<(ChannelId, &ChannelData)> {
        let channel_id = *self.channel_type_ids.get(&typeid)?;
        Some((channel_id, self.get_from_id(channel_id).unwrap()))
    }

    /// Returns a reference to the channel configuration.
    pub fn get_from_id(&self, id: ChannelId) -> Option<&ChannelData> {
        self.channel_data.get(Into::<usize>::into(id))
    }

    /// Returns whether the channel exists.
    pub fn channel_exists(&self, id: ChannelId) -> bool {
        self.channel_data.len() >= Into::<usize>::into(id)
    }

    /// Returns how many channels currently exist.
    pub fn channel_count(&self) -> u32 {
        self.channel_count
    }

    /// Returns an iterator of all existing channel ids.
    pub fn channel_ids(&self) -> impl Iterator<Item = ChannelId> {
        (0..self.channel_count).into_iter()
        .map(|f| ChannelId::try_from(f).unwrap())
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self {
            channel_count: 0,
            channel_type_ids: BTreeMap::new(),
            channel_data: vec![],
        }
    }
}

/// Channel information generated when `add_channel` is run.
pub struct ChannelData {
    /// The channel's `TypeId`.
    pub type_id: TypeId,
    /// The channel's `TypePath` (from `bevy_reflect`)
    pub type_path: &'static str,

    /// The channel's sequential ID assigned by the registry.
    pub channel_id: ChannelId,
    /// Entity ID of the channel's entity representation.
    pub entity_id: Entity,

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