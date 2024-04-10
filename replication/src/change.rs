use std::marker::PhantomData;
use bevy::{prelude::*, ecs::component::ComponentTicks};
use crate::*;

/// Metadata about network-replicated types.
pub struct ReplicateMeta<T: Replicable> {
    pub(crate) changes: NetworkChangeDetectionInner,
    phantom: PhantomData<T>,
}

impl<T: ReplicableResource> Resource for ReplicateMeta<T> {}

impl<T: ReplicableComponent> Component for ReplicateMeta<T> {
    type Storage = T::Storage;
}

/// Change detection state for network-replicated types.
pub(crate) struct NetworkChangeDetectionInner {
    pub(crate) this: ComponentTicks,
    pub(crate) other: ComponentTicks,
}