use std::marker::PhantomData;
use bevy::prelude::*;
use crate::*;

/// When added to the [`World`], replicates the resource `T`.
#[derive(Debug, Resource, Default)]
pub struct ReplicatedResource<T: ReplicableResource> {
    /// See [`ReplicationPause`]'s documentation.
    pub paused: ReplicationPause,
    pub(crate) computed: bool,
    phantom: PhantomData<T>,
}