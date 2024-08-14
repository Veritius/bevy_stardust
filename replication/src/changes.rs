//! Change detection for replicated objects.

use std::{marker::PhantomData, ops::{Deref, DerefMut}};
use bevy::{ecs::{component::{StorageType, Tick}, query::{QueryData, ReadOnlyQueryData, WorldQuery}}, prelude::*};

/// An extension of change detection for use in replication.
pub trait DetectNetChanges: DetectChanges {
    /// Returns `true` if the value was last changed by this application.
    fn is_changed_locally(&self) -> bool;

    /// Returns `true` if the value was last changed by a remote connection.
    fn is_changed_remotely(&self) -> bool;

    /// Returns the last time the value was changed by a remote connection.
    fn last_changed_remotely(&self) -> Option<Tick>;
}

/// Change detection state.
#[derive(Debug, Clone)]
struct ReplicationTicks {
    remote: Option<Tick>,
}

impl ReplicationTicks {
    pub fn is_changed_locally(
        &self,
        local_last_change: Tick,
        // last_run: Tick,
        this_run: Tick,
    ) -> bool {
        if let Some(tick) = self.remote {
            return local_last_change.is_newer_than(tick, this_run);
        }

        return true;
    }

    pub fn is_changed_remotely(
        &self,
        local_last_change: Tick,
        // last_run: Tick,
        this_run: Tick,
    ) -> bool {
        if let Some(tick) = self.remote {
            return tick.is_newer_than(local_last_change, this_run);
        }

        return false;
    }

    /// The last time the value was changed by a remote application.
    /// Returns `None` if the value has never been changed remotely.
    pub fn last_changed_remotely(&self) -> Option<Tick> {
        self.remote
    }
}

pub(crate) struct NetChangeState<T> {
    ticks: ReplicationTicks,
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
    component: Ref<'a, T>,
    tick_state: &'a NetChangeState<T>,
    sys_ticks: SystemTickData,
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
        &self.tick_state.ticks
    }
}

impl<T> DetectChanges for NetChanges<'_, T>
where
    T: Component,
{
    #[inline]
    fn is_added(&self) -> bool {
        self.component.is_added()
    }

    #[inline]
    fn is_changed(&self) -> bool {
        self.component.is_changed()
    }

    #[inline]
    fn last_changed(&self) -> Tick {
        self.component.last_changed()
    }
}

impl<T> DetectNetChanges for NetChanges<'_, T>
where
    T: Component,
{
    #[inline]
    fn is_changed_locally(&self) -> bool {
        self.tick_state.ticks.is_changed_locally(
            self.component.last_changed(),
            // self.sys_ticks.last_run,
            self.sys_ticks.this_run,
        )
    }

    #[inline]
    fn is_changed_remotely(&self) -> bool {
        self.tick_state.ticks.is_changed_remotely(
            self.component.last_changed(),
            // self.sys_ticks.last_run,
            self.sys_ticks.this_run,
        )
    }

    #[inline]
    fn last_changed_remotely(&self) -> Option<Tick> {
        self.tick_state.ticks.last_changed_remotely()
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
    tick_state: &'a mut NetChangeState<T>,
    sys_ticks: SystemTickData,
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
        // Set change ticks
        self.component.set_changed();
        self.tick_state.ticks.remote = Some(self.sys_ticks.this_run);

        // Return the component
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
        &self.tick_state.ticks
    }
}

impl<T> DetectChanges for NetChangesMut<'_, T>
where
    T: Component,
{
    #[inline]
    fn is_added(&self) -> bool {
        self.component.is_added()
    }

    #[inline]
    fn is_changed(&self) -> bool {
        self.component.is_changed()
    }

    #[inline]
    fn last_changed(&self) -> Tick {
        self.component.last_changed()
    }
}

impl<T> DetectNetChanges for NetChangesMut<'_, T>
where
    T: Component,
{
    #[inline]
    fn is_changed_locally(&self) -> bool {
        self.tick_state.ticks.is_changed_locally(
            self.component.last_changed(),
            // self.sys_ticks.last_run,
            self.sys_ticks.this_run,
        )
    }

    #[inline]
    fn is_changed_remotely(&self) -> bool {
        self.tick_state.ticks.is_changed_remotely(
            self.component.last_changed(),
            // self.sys_ticks.last_run,
            self.sys_ticks.this_run,
        )
    }

    #[inline]
    fn last_changed_remotely(&self) -> Option<Tick> {
        self.tick_state.ticks.last_changed_remotely()
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

// A hack to get tick state in a query
#[derive(Clone, Copy)]
struct SystemTickData {
    last_run: Tick,
    this_run: Tick,
}

unsafe impl WorldQuery for SystemTickData {
    type Item<'a> = Self;
    type Fetch<'a> = Self;
    type State = ();

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::Item<'wlong>) -> Self::Item<'wshort> {
        item
    }

    unsafe fn init_fetch<'w>(
        _world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell<'w>,
        _state: &Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Fetch<'w> {
        Self {
            last_run,
            this_run,
        }
    }

    const IS_DENSE: bool = true;

    unsafe fn set_archetype<'w>(
        _fetch: &mut Self::Fetch<'w>,
        _state: &Self::State,
        _archetype: &'w bevy::ecs::archetype::Archetype,
        _table: &'w bevy::ecs::storage::Table,
    ) {
        // Do nothing
    }

    unsafe fn set_table<'w>(
        _fetch: &mut Self::Fetch<'w>,
        _state: &Self::State,
        _table: &'w bevy::ecs::storage::Table,
    ) {
        // Do nothing
    }

    unsafe fn fetch<'w>(
        fetch: &mut Self::Fetch<'w>,
        _entity: Entity,
        _table_row: bevy::ecs::storage::TableRow,
    ) -> Self::Item<'w> {
        Self {
            last_run: fetch.last_run,
            this_run: fetch.this_run,
        }
    }

    fn update_component_access(
        _state: &Self::State,
        _access: &mut bevy::ecs::query::FilteredAccess<bevy::ecs::component::ComponentId>,
    ) {
        // Do nothing.
    }

    fn init_state(
        _world: &mut World,
    ) -> Self::State {
        ()
    }

    fn get_state(
        _components: &bevy::ecs::component::Components,
    ) -> Option<Self::State> {
        Some(())
    }

    fn matches_component_set(
        _state: &Self::State,
        _set_contains_id: &impl Fn(bevy::ecs::component::ComponentId) -> bool,
    ) -> bool {
        false
    }
}

unsafe impl QueryData for SystemTickData {
    type ReadOnly = Self;
}

unsafe impl ReadOnlyQueryData for SystemTickData {}