use std::marker::PhantomData;
use bevy::ecs::component::ComponentTicks;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::messaging::*;
use crate::plugin::*;
use crate::traits::*;

/// Trait for components that can be replicated.
/// Automatically implemented for types that satisfy the requirements.
pub trait ReplicableComponent: Component + Replicable {}
impl<T> ReplicableComponent for T where T: Component + Replicable {}

/// Enables replicating the component `T`.
pub struct ReplicateComponentPlugin<T: ReplicableComponent> {
    /// Message channel configuration.
    pub channel: ReplicationChannelConfiguration,

    #[doc(hidden)]
    pub phantom: PhantomData<T>,
}

impl<T: ReplicableComponent> Plugin for ReplicateComponentPlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ReplicationPlugin>() {
            panic!("ReplicationPlugin must be added before ReplicateComponentPlugin")
        }

        app.register_type::<Replicated>();
        app.register_type::<ReplicateDescendants>();      

        app.add_channel::<ReplicationData<T>>(ChannelConfiguration {
            reliable: self.channel.reliable,
            ordered: OrderingGuarantee::Sequenced,
            fragmented: true,
            priority: self.channel.priority,
        });
    }
}
/// Entities with this component will be replicated.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Replicated;

#[derive(Component)]
pub(crate) struct ReplicationTicks<T: ReplicableComponent> {
    pub inner: ComponentTicks,
    phantom: PhantomData<T>,
}

/// The descendants of this entity will be replicated, as long as the entity also has [`Replicated`].
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ReplicateDescendants;