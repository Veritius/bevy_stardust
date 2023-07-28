use std::{collections::BTreeMap, any::TypeId};
use bevy::prelude::{Resource, Entity};
use super::id::{Channel, ChannelId, CHANNEL_ID_LIMIT};

#[derive(Resource)]
pub struct ChannelRegistry {
    channel_count: u32,
    channel_type_map: BTreeMap<TypeId, ChannelId>,
    entity_map: BTreeMap<ChannelId, Entity>,
}

impl ChannelRegistry {
    pub(super) fn register_channel<T: Channel>(&mut self, entity: Entity) -> ChannelId {
        if self.channel_count >= CHANNEL_ID_LIMIT {
            panic!("Exceeded channel limit of 2^24 (how did you manage to do this?)");
        }
        
        let type_id = TypeId::of::<T>();
        let channel_id = ChannelId::from(self.channel_count - 1);
        self.channel_type_map.insert(type_id, channel_id);
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

    /// Gets the ID of an entity used to store channel configuration data.
    pub fn get_from_id(&self, id: ChannelId) -> Option<Entity> {
        self.entity_map.get(&id).cloned()
    }

    pub fn channel_exists(&self, id: ChannelId) -> bool {
        self.entity_map.contains_key(&id)
    }

    /// Returns how many channels currently exist.
    pub fn channel_count(&self) -> u32 {
        self.channel_count
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self {
            channel_count: 0,
            channel_type_map: BTreeMap::new(),
            entity_map: BTreeMap::new(),
        }
    }
}