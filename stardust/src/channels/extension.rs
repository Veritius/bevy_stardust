//! Adds `register_channel` to the `App`.

use std::any::TypeId;
use bevy::prelude::*;
use crate::{channels::outgoing::*, protocol::NetworkHashAppExt};
use self::private::Sealed;
use super::{id::Channel, config::ChannelData, registry::ChannelRegistry};

// Make it impossible to implement ChannelSetupAppExt
mod private {
    pub trait Sealed {}
    impl Sealed for bevy::prelude::App {}
}

/// Adds channel-related functions to the `App`. Can't be implemented.
pub trait ChannelSetupAppExt: Sealed {
    /// Registers a channel with type `T` and the config and components given.
    fn register_channel<T: Channel>(&mut self, components: impl Bundle + std::hash::Hash);
}

impl ChannelSetupAppExt for App {
    fn register_channel<C: Channel>(
        &mut self,
        components: impl Bundle + std::hash::Hash,
    ) {
        // Change hash value
        self.net_hash_value(("channel", C::type_path(), &components));

        // Create config entity and get registry type
        let entity_id = self.world.spawn(components).id();
        let mut registry = self.world.resource_mut::<ChannelRegistry>();

        // Create storage location on heap and register channel to registry
        let store = OutgoingOctetStringsUntyped::new();
        let channel_id = registry.register_channel::<C>(entity_id, store.clone());
        self.insert_resource(OutgoingNetworkMessages::<C>::new(store));

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