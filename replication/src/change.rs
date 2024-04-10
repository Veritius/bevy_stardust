use std::marker::PhantomData;
use bevy::ecs::{component::ComponentTicks, query::QueryFilter};
use crate::*;

/// Change tracking for changes over the network.
pub struct ReplicateMeta<T: Replicable> {
    pub(crate) changes: NetworkChangeDetectionInner,
    phantom: PhantomData<T>,
}

/// Change detection state for network-replicated types.
pub struct NetworkChangeDetectionInner {
    pub(crate) this: ComponentTicks,
    pub(crate) other: ComponentTicks,
}

struct NetChanged<T: Replicable> {
    phantom: PhantomData<T>,
}