use std::marker::PhantomData;
use bevy::{ecs::query::{WorldQuery, QueryFilter}, prelude::*};
use crate::ReplicableComponent;

/// Entities with this component will be replicated.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ReplicateEntity;

/// The descendants of this entity will be replicated, as long as the entity also has [`Replicated`].
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ReplicateDescendants;

/// Query filter for changes on network-replicated entities.
pub struct NetChanged<T: ReplicableComponent> {
    phantom: PhantomData<T>,
}

unsafe impl<T: ReplicableComponent> WorldQuery for NetChanged<T> {
    type Item<'a> = ();
    type Fetch<'a> = ();
    type State = ();

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::Item<'wlong>) -> Self::Item<'wshort> {
        todo!()
    }

    unsafe fn init_fetch<'w>(
        world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell<'w>,
        state: &Self::State,
        last_run: bevy::ecs::component::Tick,
        this_run: bevy::ecs::component::Tick,
    ) -> Self::Fetch<'w> {
        todo!()
    }

    const IS_DENSE: bool = false;

    unsafe fn set_archetype<'w>(
        fetch: &mut Self::Fetch<'w>,
        state: &Self::State,
        archetype: &'w bevy::ecs::archetype::Archetype,
        table: &'w bevy::ecs::storage::Table,
    ) {
        todo!()
    }

    unsafe fn set_table<'w>(fetch: &mut Self::Fetch<'w>, state: &Self::State, table: &'w bevy::ecs::storage::Table) {
        todo!()
    }

    unsafe fn fetch<'w>(
        fetch: &mut Self::Fetch<'w>,
        entity: Entity,
        table_row: bevy::ecs::storage::TableRow,
    ) -> Self::Item<'w> {
        todo!()
    }

    fn update_component_access(state: &Self::State, access: &mut bevy::ecs::query::FilteredAccess<bevy::ecs::component::ComponentId>) {
        todo!()
    }

    fn init_state(world: &mut World) -> Self::State {
        todo!()
    }

    fn get_state(world: &World) -> Option<Self::State> {
        todo!()
    }

    fn matches_component_set(
        state: &Self::State,
        set_contains_id: &impl Fn(bevy::ecs::component::ComponentId) -> bool,
    ) -> bool {
        todo!()
    }
}

impl<T: ReplicableComponent> QueryFilter for NetChanged<T> {
    const IS_ARCHETYPAL: bool = false;

    unsafe fn filter_fetch(
        fetch: &mut Self::Fetch<'_>,
        entity: Entity,
        table_row: bevy::ecs::storage::TableRow,
    ) -> bool {
        todo!()
    }
}