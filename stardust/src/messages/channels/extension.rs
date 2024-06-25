//! Adds `add_channel` to the `App`.

use bevy::app::App;
use super::config::ChannelConfiguration;
use super::{id::Channel, ChannelRegistryMut};

mod sealed {
    pub trait Sealed {}
    impl Sealed for bevy::app::App {}
}

/// Adds channel-related functions to the `App`.
pub trait ChannelSetupAppExt: sealed::Sealed {
    /// Registers a channel with type `C` and the config and components given.
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

        // Add to registry
        let mut registry = self.world.resource_mut::<ChannelRegistryMut>();
        registry.0.register_channel::<C>(config);
    }
}