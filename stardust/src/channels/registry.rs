//! The channel registry.

use std::{collections::BTreeMap, any::TypeId};
use bevy::{prelude::Resource, ecs::component::ComponentId};
use crate::{octets::varints::u24, prelude::ChannelConfiguration};
use super::id::{Channel, ChannelId, CHANNEL_ID_LIMIT};

pub(super) struct ChannelWorldMeta {
    pub incoming_events: ComponentId,
    pub outgoing_queue: ComponentId,
}

/// Channel information generated when `register_channel` is run.
pub struct ChannelData {
    /// The channel's `TypeId`.
    pub type_id: TypeId,
    /// The channel's `TypePath` (from `bevy_reflect`)
    pub type_path: &'static str,
    /// The channel's sequential ID assigned by the registry.
    pub channel_id: ChannelId,

    /// The `ComponentId` of the `Events<NetworkMessage<C>>` resource, where `C` is the channel.
    pub incoming_events_component_id: ComponentId,
    /// The `ComponentId` of the `OutgoingNetworkMessages<C>` resource, where `C` is the channel.
    pub outgoing_queue_component_id: ComponentId,

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

/// Stores information related to type ids.
#[derive(Resource)]
pub struct ChannelRegistry {
    channel_count: u32,
    channel_type_map: BTreeMap<TypeId, ChannelId>,
    channel_data_map: BTreeMap<ChannelId, ChannelData>,
}

impl ChannelRegistry {
    pub(in crate) fn new() -> Self {
        Self {
            channel_count: 0,
            channel_type_map: BTreeMap::new(),
            channel_data_map: BTreeMap::new(),
        }    
    }

    pub(super) fn register_channel<C: Channel>(
        &mut self,
        config: ChannelConfiguration,
        meta: ChannelWorldMeta,
    ) -> ChannelId {
        // Check we don't overrun the channel ID
        if self.channel_count >= CHANNEL_ID_LIMIT {
            panic!("Exceeded channel limit of {}", CHANNEL_ID_LIMIT);
        }
        
        // Check the channel doesn't already exist
        let type_id = TypeId::of::<C>();
        let type_path = C::type_path();
        if self.channel_type_map.get(&type_id).is_some() {
            panic!("A channel was registered twice: {type_path}");
        }

        // Add to map
        let channel_id = ChannelId::try_from(self.channel_count).unwrap();
        self.channel_type_map.insert(type_id, channel_id);
        self.channel_data_map.insert(channel_id, ChannelData {
            type_id,
            type_path,
            channel_id,

            incoming_events_component_id: meta.incoming_events,
            outgoing_queue_component_id: meta.outgoing_queue,

            config,
        });
        self.channel_count += 1;

        channel_id
    }

    /// Returns the ChannelId and a reference to the ChannelConfig
    pub fn get_from_type<C: Channel>(&self) -> Option<(ChannelId, &ChannelData)> {
        let type_id = TypeId::of::<C>();
        let channel_id = *self.channel_type_map.get(&type_id)?;
        let config = self.channel_data_map.get(&channel_id)?;
        Some((channel_id, config))
    }

    /// Returns a reference to the channel configuration.
    pub fn get_from_id(&self, id: ChannelId) -> Option<&ChannelData> {
        self.channel_data_map.get(&id)
    }

    /// Returns whether the channel exists.
    pub fn channel_exists(&self, id: ChannelId) -> bool {
        self.channel_data_map.contains_key(&id)
    }

    /// Returns how many channels currently exist.
    pub fn channel_count(&self) -> u32 {
        self.channel_count
    }

    /// Returns an iterator of all existing channel ids.
    pub fn channel_ids(&self) -> impl Iterator<Item = ChannelId> {
        (0..self.channel_count).into_iter()
        .map(|f| ChannelId(TryInto::<u24>::try_into(f).unwrap()))
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self {
            channel_count: 0,
            channel_type_map: BTreeMap::new(),
            channel_data_map: BTreeMap::new(),
        }
    }
}