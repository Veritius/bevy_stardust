//! Adds `add_channel` to the `App`.

use bevy::prelude::*;
use crate::{protocol::ProtocolIdAppExt, prelude::ChannelConfiguration};
use super::{
    id::{Channel, ChannelMarker},
    registry::ChannelRegistry,
    incoming::IncomingMessages,
    outgoing::OutgoingMessages
};

mod sealed {
    pub trait Sealed {}
    impl Sealed for bevy::prelude::App {}
}

/// Adds channel-related functions to the `App`.
pub trait ChannelSetupAppExt: sealed::Sealed {
    /// Registers a channel with type `T` and the config and components given.
    /// 
    /// ```ignore
    /// // Simple example
    /// app.add_channel::<MyChannel>(ChannelConfiguration {
    ///     reliable: ChannelReliability::Reliable,
    ///     ordering: ChannelOrdering::Ordered,
    ///     fragmentation: ChannelFragmentation::Disabled,
    ///     compression: ChannelCompression::Disabled,
    ///     validation: MessageValidation::Disabled,
    /// });
    /// ```
    fn add_channel<C: Channel>(&mut self, config: ChannelConfiguration);
}

impl ChannelSetupAppExt for App {
    fn add_channel<C: Channel>(
        &mut self,
        config: ChannelConfiguration,
    ) {
        // Change hash value
        self.net_hash_value(("channel", C::type_path(), &config));
        
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