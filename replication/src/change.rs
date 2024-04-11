use std::marker::PhantomData;
use std::cell::UnsafeCell;
use bevy::{ecs::{component::*, ptr::*, query::*, storage::*}, prelude::*};
use crate::*;

/// Metadata about network-replicated types.
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

/// Query filter for changes on network-replicated entities.
pub struct NetChanged<T: ReplicableComponent> {
    phantom: PhantomData<T>,
}

#[doc(hidden)]
pub struct NetChangedFetch<'w, T: ReplicableComponent> {
    table_components: Option<ThinSlicePtr<'w, UnsafeCell<NetChanges<T>>>>,
    sparse_set: Option<&'w ComponentSparseSet>,
    last_run: Tick,
    this_run: Tick,
}

impl<T: ReplicableComponent> Clone for NetChangedFetch<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ReplicableComponent> Copy for NetChangedFetch<'_, T> {}

unsafe impl<T: ReplicableComponent> WorldQuery for NetChanged<T> {
    type Item<'w> = bool;
    type Fetch<'w> = NetChangedFetch<'w, T>;
    type State = ComponentId;

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::Item<'wlong>) -> Self::Item<'wshort> {
        item
    }

    #[inline]
    unsafe fn init_fetch<'w>(
        world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell<'w>,
        &component_id: &Self::State,
        last_run: bevy::ecs::component::Tick,
        this_run: bevy::ecs::component::Tick,
    ) -> Self::Fetch<'w> {
        Self::Fetch::<'w> {
            table_components: None,
            sparse_set: (T::Storage::STORAGE_TYPE == StorageType::SparseSet).then(|| {
                debug_unchecked_unwrap(world.storages().sparse_sets.get(component_id))
            }),
            last_run,
            this_run,
        }
    }

    const IS_DENSE: bool = {
        match T::Storage::STORAGE_TYPE {
            StorageType::Table => true,
            StorageType::SparseSet => false,
        }
    };

    unsafe fn set_archetype<'w>(
        fetch: &mut Self::Fetch<'w>,
        state: &Self::State,
        _archetype: &'w bevy::ecs::archetype::Archetype,
        table: &'w bevy::ecs::storage::Table,
    ) {
        if Self::IS_DENSE {
            Self::set_table(fetch, state, table);
        }
    }

    unsafe fn set_table<'w>(
        fetch: &mut Self::Fetch<'w>,
        &component_id: &Self::State,
        table: &'w bevy::ecs::storage::Table
    ) {
        fetch.table_components = Some(
            debug_unchecked_unwrap(table.get_column(component_id)).get_data_slice().into()
        )
    }

    unsafe fn fetch<'w>(
        fetch: &mut Self::Fetch<'w>,
        entity: Entity,
        table_row: bevy::ecs::storage::TableRow,
    ) -> Self::Item<'w> {
        let component = match T::Storage::STORAGE_TYPE {
            StorageType::Table => debug_unchecked_unwrap(fetch.table_components).get(table_row.as_usize()).deref(),
            StorageType::SparseSet => debug_unchecked_unwrap(debug_unchecked_unwrap(fetch.sparse_set).get(entity)).deref(),
        };

        return component.changes.this_tick.last_changed_tick().is_newer_than(fetch.last_run, fetch.this_run);
    }

    fn update_component_access(
        &component_id: &Self::State,
        access: &mut FilteredAccess<ComponentId>
    ) {
        if access.access().has_write(component_id) {
            panic!("$state_name<{}> conflicts with a previous access in this query. Shared access cannot coincide with exclusive access.", std::any::type_name::<T>());
        }
        access.add_read(component_id);
    }

    fn init_state(world: &mut World) -> Self::State {
        world.init_component::<NetChanges<T>>()
    }

    fn get_state(world: &World) -> Option<Self::State> {
        world.component_id::<NetChanges<T>>()
    }

    fn matches_component_set(
        &component_id: &Self::State,
        set_contains_id: &impl Fn(ComponentId) -> bool,
    ) -> bool {
        set_contains_id(component_id)
    }
}

impl<T: ReplicableComponent> QueryFilter for NetChanged<T> {
    const IS_ARCHETYPAL: bool = false;

    unsafe fn filter_fetch(
        fetch: &mut Self::Fetch<'_>,
        entity: Entity,
        table_row: bevy::ecs::storage::TableRow,
    ) -> bool {
        Self::fetch(fetch, entity, table_row)
    }
}

#[inline]
fn debug_unchecked_unwrap<T>(item: Option<T>) -> T {
    #[cfg(debug_assertions)]
    { item.unwrap() }
    #[cfg(not(debug_assertions))]
    { item.unwrap_unchecked() }
}