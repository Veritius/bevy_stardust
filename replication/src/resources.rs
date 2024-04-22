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
    }
}