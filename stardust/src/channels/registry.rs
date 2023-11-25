//! The channel registry.

use std::{collections::BTreeMap, any::TypeId, sync::{Arc, RwLock}, marker::PhantomData};
use bevy::prelude::Resource;
use crate::{messages::outgoing::OutgoingMessageQueueInternal, octets::varints::u24, prelude::{ChannelData, ChannelConfiguration}};
use super::id::{Channel, ChannelId, CHANNEL_ID_LIMIT};

/// Stores information related to type ids.
#[derive(Resource)]
pub struct ChannelRegistry {
    channel_count: u32,
    channel_type_map: BTreeMap<TypeId, ChannelId>,
    outgoing_arc_map: BTreeMap<ChannelId, Arc<RwLock<OutgoingMessageQueueInternal>>>,
    channel_data_map: BTreeMap<ChannelId, ChannelData>,
}

impl ChannelRegistry {
    pub(in crate) fn new() -> Self {
        Self {
            channel_count: 0,
            channel_type_map: BTreeMap::new(),
            outgoing_arc_map: BTreeMap::new(),
            channel_data_map: BTreeMap::new(),
        }    
    }

    pub(super) fn register_channel<C: Channel>(
        &mut self,
        config: ChannelConfiguration,
        untyped_store: Arc<RwLock<OutgoingMessageQueueInternal>>
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
        self.outgoing_arc_map.insert(channel_id, untyped_store);
        self.channel_data_map.insert(channel_id, ChannelData {
            type_id,
            type_path,
            channel_id,
            config,
            phantom: PhantomData
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

    pub(crate) fn get_outgoing_arc_map(&self) -> &BTreeMap<ChannelId, Arc<RwLock<OutgoingMessageQueueInternal>>> {
        &self.outgoing_arc_map
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self {
            channel_count: 0,
            channel_type_map: BTreeMap::new(),
            outgoing_arc_map: BTreeMap::new(),
            channel_data_map: BTreeMap::new(),
        }
    }
}