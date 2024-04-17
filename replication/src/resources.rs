use std::{marker::PhantomData, ops::{Deref, DerefMut}};
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_stardust::prelude::*;
use crate::prelude::*;

#[derive(Default)]
pub(crate) struct ResourceReplicationData<T: ReplicableResource>(PhantomData<T>);

/// Enables replicating the resource `T`.
/// 
/// This plugin must be added before [`StardustPlugin`].
/// Implicitly adds [`ReplicationPlugin`] if not present.
pub struct ReplicateResourcePlugin<T: ReplicableResource> {
    /// If replication data should be sent reliably.
    pub reliability: ReliabilityGuarantee,

    /// The priority of the resource to replicate.
    /// Higher priority items will be replicated first.
    pub priority: u32,

    #[doc(hidden)]
    pub phantom: PhantomData<T>,
}

impl<T: ReplicableResource> Plugin for ReplicateResourcePlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<CoreReplicationPlugin>() {
            app.add_plugins(CoreReplicationPlugin);
        }

        app.add_channel::<ResourceReplicationData<T>>(ChannelConfiguration {
            reliable: self.reliability,
            ordered: OrderingGuarantee::Sequenced,
            fragmented: true,
            priority: self.priority,
        });
    }
}

/// When added to the [`World`], replicates the resource `T`.
#[derive(Debug, Resource, Default)]
pub struct ReplicateResource<T: ReplicableResource> {
    /// When `true`, stops synchronising data, but doesn't remove the resource.
    /// To remove the resource from all connections, remove this resource.
    pub paused: bool,
    phantom: PhantomData<T>,
}

/// Immutable [`Resource`] access with replication metadata.
#[derive(SystemParam)]
pub struct NetRes<'w, T: ReplicableResource> {
    /// Inner resource data.
    pub data: Res<'w, T>,
    /// Change detection data.
    pub changes: Res<'w, NetChanges<T>>,
}

impl<'w, T: ReplicableResource> NetRes<'w, T> {

}

impl<T: ReplicableResource> Deref for NetRes<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// Mutable [`Resource`] access with replication metadata.
#[derive(SystemParam)]
pub struct NetResMut<'w, T: ReplicableResource> {
    /// Inner resource data.
    pub data: ResMut<'w, T>,
    /// Change detection data.
    pub changes: Res<'w, NetChanges<T>>,
}

impl<'w, T: ReplicableResource> NetResMut<'w, T> {

}

impl<T: ReplicableResource> Deref for NetResMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: ReplicableResource> DerefMut for NetResMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}