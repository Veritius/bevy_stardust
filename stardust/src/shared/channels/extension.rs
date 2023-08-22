use std::any::TypeId;
use bevy::prelude::*;
use crate::shared::{channels::outgoing::{OutgoingOctetStringsUntyped, OutgoingOctetStrings}, hashdiff::NetworkHashAppExt};
use super::{id::Channel, components::ChannelData, registry::ChannelRegistry};

pub trait ChannelSetupAppExt {
    /// Registers a channel with type `T` and the config and components given.
    fn register_channel<T: Channel>(&mut self, components: impl Bundle + std::hash::Hash);
}

impl ChannelSetupAppExt for App {
    fn register_channel<C: Channel>(
        &mut self,
        components: impl Bundle + std::hash::Hash,
    ) {
        // Create config entity and get registry type
        let entity_id = self.world.spawn(components).id();
        let mut registry = self.world.resource_mut::<ChannelRegistry>();

        // Create storage location on heap and register channel to registry
        let store = OutgoingOctetStringsUntyped::new();
        let channel_id = registry.register_channel::<C>(entity_id, store.clone());
        self.insert_resource(OutgoingOctetStrings::<C>::new(store));

        // Change hash value
        self.add_net_hash_value(("channel", C::type_path())); // TODO: Check components

        // Spawn config entity
        let type_id = TypeId::of::<C>();
        self.world.entity_mut(entity_id).insert(ChannelData {
            type_id,
            type_path: C::type_path(),
            channel_id,
        });
        
        // Log addition at debug level
        debug!("Channel registered with type ID {:?} on channel ID {:?} with config entity {:?} ", type_id, channel_id, entity_id);
    }
}