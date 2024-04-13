mod queries;

pub use queries::NetChanged;

use std::marker::PhantomData;
use bevy::{prelude::*, ecs::component::*};
use crate::prelude::*;

/// Change detection tracking for network replicated types.
pub struct NetChanges<T: Replicable> {
    pub(crate) changes: NetworkChangeDetectionInner,
    phantom: PhantomData<T>,
}

impl<T: ReplicableResource> Resource for NetChanges<T> {}

impl<T: ReplicableComponent> Component for NetChanges<T> {
    type Storage = T::Storage;
}

/// Change detection state for network-replicated types.
pub(crate) struct NetworkChangeDetectionInner {
    pub(crate) this_tick: ComponentTicks,
}