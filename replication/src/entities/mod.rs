mod hierarchy;
mod ids;

pub(crate) use ids::*;

pub use hierarchy::*;

use std::marker::PhantomData;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::prelude::*;

/// Query filter for entities that are replicated over the network.
pub type Replicated = With<ReplicateEntity>;

/// Entities with this component will be replicated.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ReplicateEntity;

#[derive(Default)]
pub(crate) struct ComponentReplicationData<T: ReplicableComponent>(PhantomData<T>);

/// Enables replicating the component `T`.
/// 
/// This plugin must be added before [`StardustPlugin`].
/// Implicitly adds [`ReplicationPlugin`] if not present.
pub struct ReplicateComponentPlugin<T: ReplicableComponent> {
    /// If replication data should be sent reliably.
    pub reliability: ReliabilityGuarantee,

    /// The priority of the resource to replicate.
    /// Higher priority items will be replicated first.
    pub priority: u32,

    #[doc(hidden)]
    pub phantom: PhantomData<T>,
}

impl<T: ReplicableComponent> Plugin for ReplicateComponentPlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<CoreReplicationPlugin>() {
            app.add_plugins(CoreReplicationPlugin);
        }

        app.register_type::<ReplicateEntity>();
        app.register_type::<ReplicateHierarchy>();

        app.add_channel::<ComponentReplicationData<T>>(ChannelConfiguration {
            reliable: self.reliability,
            ordered: OrderingGuarantee::Sequenced,
            fragmented: true,
            priority: self.priority,
        });
    }
}