use std::{marker::PhantomData, ops::{Deref, DerefMut}};
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_stardust::prelude::*;
use crate::{prelude::*, serialisation::SerialisationFunctions};

#[derive(Default)]
pub(crate) struct ResourceReplicationChannel<T: Resource>(PhantomData<T>);

/// Enables replicating the resource `T`.
/// 
/// This plugin must be added before [`StardustPlugin`].
/// Implicitly adds [`ReplicationPlugin`] if not present.
pub struct ResourceReplicationPlugin<T: Resource> {
    /// Functions used to serialise and deserialize `T`.
    /// See the [`SerialisationFunctions`] documentation for more information.
    pub serialisation: SerialisationFunctions<T>,

    /// The priority of network messages for replicating `T`.
    pub message_priority: u32,

    #[doc(hidden)]
    pub phantom: PhantomData<T>,
}

impl<T: Resource> Plugin for ResourceReplicationPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_channel::<ResourceReplicationChannel<T>>(ChannelConfiguration {
            reliable: ReliabilityGuarantee::Reliable,
            ordered: OrderingGuarantee::Sequenced,
            fragmented: true,
            priority: self.message_priority,
        });

        app.add_systems(PostUpdate, crate::change::undirty_resource_system::<T>
            .in_set(PostUpdateReplicationSystems::ClearDirty));
    }
}

/// Immutable [`Resource`] access with replication metadata.
#[derive(SystemParam)]
pub struct NetRes<'w, T: Resource> {
    /// Inner resource data.
    pub data: Res<'w, T>,
    /// Change detection data.
    pub changes: Res<'w, NetChanges<T>>,
}

impl<'w, T: Resource> NetRes<'w, T> {

}

impl<T: Resource> Deref for NetRes<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// Mutable [`Resource`] access with replication metadata.
#[derive(SystemParam)]
pub struct NetResMut<'w, T: Resource> {
    /// Inner resource data.
    pub data: ResMut<'w, T>,
    /// Change detection data.
    pub changes: Res<'w, NetChanges<T>>,
}

impl<'w, T: Resource> NetResMut<'w, T> {

}

impl<T: Resource> Deref for NetResMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: Resource> DerefMut for NetResMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}