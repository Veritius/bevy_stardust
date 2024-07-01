//! Adds `add_channel` to the `App`.

use bevy::app::App;
use super::config::ChannelConfiguration;
use super::ChannelId;
use super::{id::Channel, ChannelRegistryBuilder};

mod sealed {
    pub trait Sealed {}
    impl Sealed for bevy::app::App {}
}

/// Adds channel-related functions to the `App`.
pub trait ChannelSetupAppExt: sealed::Sealed {
    /// Registers a channel with type `C` and the config and components given.
    /// Returns the sequential `ChannelId` now associated with the channel.
    fn add_channel<C: Channel>(&mut self, config: ChannelConfiguration) -> ChannelId;
}

impl ChannelSetupAppExt for App {
    fn add_channel<C: Channel>(
        &mut self,
        config: ChannelConfiguration,
    ) -> ChannelId {
        // Get the registry
        let mut registry = self.world.get_resource_mut::<ChannelRegistryBuilder>()
            .expect("Cannot add channels after plugin cleanup");

        // Add to registry
        return registry.0.register_channel::<C>(config);
    }
}