use std::marker::PhantomData;
use bevy::prelude::*;
use crate::*;

/// When added to the [`World`], replicates the resource `T`.
#[derive(Debug, Resource, Default)]
pub struct ReplicatedResource<T: ReplicableResource> {
    /// See [`ReplicationState`]'s documentation.
    pub state: ReplicationState,
    phantom: PhantomData<T>,
}