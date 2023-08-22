//! The channel registry.

use std::{collections::BTreeMap, any::TypeId, sync::{Arc, RwLock}};
use bevy::prelude::{Resource, Entity};
use super::{id::{Channel, ChannelId, CHANNEL_ID_LIMIT}, outgoing::OutgoingOctetStringsUntyped};

#[derive(Resource)]
pub struct ChannelRegistry {
    channel_count: u32,
    channel_type_map: BTreeMap<TypeId, ChannelId>,
    outgoing_arc_map: BTreeMap<ChannelId, Arc<RwLock<OutgoingOctetStringsUntyped>>>,
    entity_map: BTreeMap<ChannelId, Entity>,
}

impl ChannelRegistry {
    pub(in crate::shared) fn new() -> Self {
        Self {
            channel_count: 0,
            channel_type_map: BTreeMap::new(),
            outgoing_arc_map: BTreeMap::new(),
            entity_map: BTreeMap::new(),
        }    
    }

    pub(super) fn register_channel<T: Channel>(
        &mut self,
        entity: Entity,
        untyped_store: Arc<RwLock<OutgoingOctetStringsUntyped>>
    ) -> ChannelId {
        if self.channel_count >= CHANNEL_ID_LIMIT {
            panic!("Exceeded channel limit of {}", CHANNEL_ID_LIMIT);
        }
        
        // Check the channel doesn't already exist
        let type_id = TypeId::of::<T>();
        if self.channel_type_map.get(&type_id).is_some() {
            panic!("A channel was registered twice: {}", T::type_path());
        }
        
        // Add to map
        let channel_id = ChannelId::try_from(self.channel_count).unwrap();
        self.channel_type_map.insert(type_id, channel_id);
        self.outgoing_arc_map.insert(channel_id, untyped_store);
        self.entity_map.insert(channel_id, entity);
        self.channel_count += 1;

        channel_id
    }

    /// Gets the numerical ChannelId and entity for `T` if it exists.
    pub fn get_from_type<T: Channel>(&self) -> Option<(ChannelId, Entity)> {
        let type_id = TypeId::of::<T>();
        let channel_id = *self.channel_type_map.get(&type_id)?;
        let entity = *self.entity_map.get(&channel_id)?;
        Some((channel_id, entity))
    }

    /// Gets the ID of the entity used to store channel configuration data.
    pub fn get_from_id(&self, id: ChannelId) -> Option<Entity> {
        self.entity_map.get(&id).cloned()
    }

    /// Returns whether the channel exists.
    pub fn channel_exists(&self, id: ChannelId) -> bool {
        self.entity_map.contains_key(&id)
    }

    /// Returns how many channels currently exist.
    pub fn channel_count(&self) -> u32 {
        self.channel_count
    }

    pub(super) fn get_outgoing_arc_map(&self) -> &BTreeMap<ChannelId, Arc<RwLock<OutgoingOctetStringsUntyped>>> {
        &self.outgoing_arc_map
    }

    pub(super) fn get_outgoing_arc(&self, id: ChannelId) -> Option<Arc<RwLock<OutgoingOctetStringsUntyped>>> {
        self.outgoing_arc_map.get(&id).cloned()
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self {
            channel_count: 0,
            channel_type_map: BTreeMap::new(),
            outgoing_arc_map: BTreeMap::new(),
            entity_map: BTreeMap::new(),
        }
    }
}