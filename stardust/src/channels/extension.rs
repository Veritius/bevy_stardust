//! Adds `add_channel` to the `App`.

use bevy_app::App;
use crate::channels::config::ChannelConfiguration;
use super::{
    id::{Channel, ChannelMarker},
    registry::ChannelRegistry,
    incoming::IncomingMessages,
    outgoing::OutgoingMessages
};

mod sealed {
    pub trait Sealed {}
    impl Sealed for bevy_app::App {}
}

/// Adds channel-related functions to the `App`.
pub trait ChannelSetupAppExt: sealed::Sealed {
    /// Registers a channel with type `T` and the config and components given.
    fn add_channel<C: Channel>(&mut self, config: ChannelConfiguration);
}

impl ChannelSetupAppExt for App {
    fn add_channel<C: Channel>(
        &mut self,
        config: ChannelConfiguration,
    ) {
        // Change hash value
        #[cfg(feature="hashing")] {
            use crate::hashing::HashingAppExt;
            self.net_hash_value("channel");
            self.net_hash_value(C::type_path());
            self.net_hash_value(&config);
        }

        // Spawn entity
        let entity = self.world.spawn((
            ChannelMarker::<C>::default(),
            IncomingMessages::default(),
            OutgoingMessages::default(),
        )).id();

        // Add to registry
        let mut registry = self.world.resource_mut::<ChannelRegistry>();
        registry.register_channel::<C>(config, entity);
    }
}