//! Change detection for replicated objects.

use std::{marker::PhantomData, ops::{Deref, DerefMut}};
use bevy::{ecs::{component::{StorageType, Tick}, query::QueryData}, prelude::*};

/// Change detection state.
#[derive(Debug, Clone)]
pub struct ReplicationTicks {
    local: Option<Tick>,
    remote: Option<Tick>,
}

impl ReplicationTicks {
    /// Returns `true` if the component has changed, and the change was done by this application.
    pub fn is_changed_locally(&self, last_run: Tick, this_run: Tick) -> bool {
        if self.local.is_none() { return false }
        self.local.unwrap().is_newer_than(last_run, this_run)
    }

    /// Returns `true` if the component has changed, and the change was done by a remote application.
    pub fn is_changed_remotely(&self, last_run: Tick, this_run: Tick) -> bool {
        if self.remote.is_none() { return false }
        self.remote.unwrap().is_newer_than(last_run, this_run)
    }

    /// The last time the value was changed by this application.
    /// Returns `None` if the value has never been changed locally.
    pub fn last_changed_locally(&self) -> Option<Tick> {
        self.local
    }

    /// The last time the value was changed by a remote application.
    /// Returns `None` if the value has never been changed remotely.
    pub fn last_changed_remotely(&self) -> Option<Tick> {
        self.remote
    }
}

pub(crate) struct NetChangeState<T> {
    pub ticks: ReplicationTicks,

    _ph1: PhantomData<T>,
}

impl<T: Component> Component for NetChangeState<T> {
    const STORAGE_TYPE: StorageType = StorageType::Table;
}

impl<T: Resource> Resource for NetChangeState<T> {}

/// Access to a component and its associated change detection state.
#[derive(QueryData)]
pub struct NetChanges<'a, T>
where
    T: Component,
{
    component: &'a T,
    ticks: &'a NetChangeState<T>,
}

impl<T> Deref for NetChanges<'_, T>
where
    T: Component,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl<T> AsRef<T> for NetChanges<'_, T>
where
    T: Component,
{
    #[inline]
    fn as_ref(&self) -> &T {
        &self.component
    }
}

impl<T> AsRef<ReplicationTicks> for NetChanges<'_, T>
where
    T: Component,
{
    #[inline]
    fn as_ref(&self) -> &ReplicationTicks {
        &self.ticks.ticks
    }
}

/// Mutable access to a component and its associated change detection state.
/// 
/// Like [`Mut`], mutation through [`DerefMut`] will set the component as changed by the local state.
pub struct NetChangesMut<'a, T>
where
    T: Component,
{
    component: Mut<'a, T>,
    ticks: &'a mut NetChangeState<T>,
}

impl<T> Deref for NetChangesMut<'_, T>
where
    T: Component,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl<T> DerefMut for NetChangesMut<'_, T>
where
    T: Component,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // TODO
        return self.component.as_mut();
    }
}

impl<T> AsRef<T> for NetChangesMut<'_, T>
where
    T: Component,
{
    fn as_ref(&self) -> &T {
        &self.component
    }
}

impl<T> AsRef<ReplicationTicks> for NetChangesMut<'_, T>
where
    T: Component,
{
    #[inline]
    fn as_ref(&self) -> &ReplicationTicks {
        &self.ticks.ticks
    }
}

/// Filters for entities that have been changed by the local application.
#[derive(QueryData)]
pub struct ChangedLocally<'a, T>
where
    T: Component,
{
    changes: NetChanges<'a, T>,
}

/// Filters for entities that have been changed by the remote application.
#[derive(QueryData)]
pub struct ChangedRemotely<'a, T>
where
    T: Component,
{
    changes: NetChanges<'a, T>,
}