//! Adds `register_channel` to the `App`.

use bevy::prelude::*;
use crate::{protocol::ProtocolIdAppExt, prelude::ChannelConfiguration};
use super::{id::Channel, registry::ChannelRegistry};

mod sealed {
    pub trait Sealed {}
    impl Sealed for bevy::prelude::App {}
}

/// Adds channel-related functions to the `App`.
pub trait ChannelSetupAppExt: sealed::Sealed {
    /// Registers a channel with type `T` and the config and components given.
    fn register_channel<T: Channel>(&mut self, config: ChannelConfiguration);
}

impl ChannelSetupAppExt for App {
    fn register_channel<C: Channel>(
        &mut self,
        config: ChannelConfiguration,
    ) {
        // Change hash value
        self.net_hash_value(("channel", C::type_path(), &config));

        let mut registry = self.world.resource_mut::<ChannelRegistry>();
        registry.register_channel::<C>(config);
    }
}