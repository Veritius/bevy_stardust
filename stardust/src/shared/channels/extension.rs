use std::any::TypeId;
use bevy::prelude::*;
use crate::shared::messages::send::OutgoingOctetStrings;

use super::{id::Channel, components::{ChannelData, ChannelConfig}, registry::ChannelRegistry};

pub trait ChannelSetupAppExt {
    /// Registers a channel with type `T` and the config and components given.
    fn register_channel<T: Channel>(&mut self, config: ChannelConfig, components: impl Bundle);
}

impl ChannelSetupAppExt for App {
    fn register_channel<T: Channel>(
        &mut self,
        config: ChannelConfig,
        components: impl Bundle,
    ) {
        let entity_id = self.world.spawn(components).id();
        let mut registry = self.world.resource_mut::<ChannelRegistry>();
        let channel_id = registry.register_channel::<T>(entity_id);

        self.insert_resource(OutgoingOctetStrings::<T>::default());
        
        let type_id = TypeId::of::<T>();
        self.world.entity_mut(entity_id).insert(ChannelData {
            config,
            type_id,
            channel_id,
        });
        
        trace!("Channel registered with ID {:?} on entity {:?}", channel_id, entity_id);
    }
}