use std::{marker::PhantomData, ops::{Deref, DerefMut}};
use bevy::{ecs::system::SystemParam, prelude::*};
use crate::*;

/// When added to the [`World`], replicates the resource `T`.
#[derive(Debug, Resource, Default)]
pub struct ReplicatedResource<T: ReplicableResource> {
    /// See [`ReplicationPause`]'s documentation.
    pub paused: ReplicationPause,
    pub(crate) computed: bool,
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