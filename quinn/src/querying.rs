use std::net::SocketAddr;
use bevy_ecs::{prelude::*, query::{QueryData, WorldQuery}};
use crate::{config::SocketConfig, endpoints::EndpointComp};
use crate::connections::ConnectionComp;

/// Type system ID of a QUIC endpoint.
/// 
/// This is **not** the component this plugin uses.
/// Instead, it's a [`WorldQuery`] that hides the inner workings while still allowing querying.
/// It can be used almost identically to how you would use a component in a query.
/// This is also why the type is unconstructable (has no variants) since it is not used by value.
/// 
/// ```no_run
/// # use bevy_ecs::prelude::*;
/// // For read-only access (works in parallel)
/// type MyQuery<'a> = Query<&'a Endpoint>;
/// 
/// // For edit access (does not work in parallel)
/// type MyMutQuery<'a> = Query<&'a mut Endpoint>;
/// 
/// // Usage example in a query
/// fn my_system(
///     query: Query<&mut Endpoint>,
/// ) {}
/// ```
pub enum Endpoint {}

#[derive(QueryData)]
pub struct EndpointRef<'w> {
    inner: Ref<'w, EndpointComp>,
}

impl<'w> EndpointRef<'w> {
    #[inline]
    pub fn socket(&'w self) -> &'w SocketConfig {
        &self.inner.socket_cfg
    }

    #[inline]
    pub fn local_addr(&'w self) -> SocketAddr {
        self.inner.local_addr()
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct EndpointMut<'w> {
    inner: Mut<'w, EndpointComp>,
}

impl<'w> EndpointMut<'w> {
    #[inline]
    pub fn socket(&'w self) -> &'w SocketConfig {
        &self.inner.socket_cfg
    }

    #[inline]
    pub fn socket_mut(&'w mut self) -> &'w mut SocketConfig {
        &mut self.inner.socket_cfg
    }

    #[inline]
    pub fn local_addr(&'w self) -> SocketAddr {
        self.inner.local_addr()
    }
}

/// Type system ID of a QUIC connection.
/// 
/// This is **not** the component this plugin uses.
/// Instead, it's a [`WorldQuery`] that hides the inner workings while still allowing querying.
/// It can be used almost identically to how you would use a component in a query.
/// This is also why the type is unconstructable (has no variants) since it is not used by value.
/// 
/// ```no_run
/// # use bevy_ecs::prelude::*;
/// // For read-only access (works in parallel)
/// type MyQuery<'a> = Query<&'a Connection>;
/// 
/// // For edit access (does not work in parallel)
/// type MyMutQuery<'a> = Query<&'a mut Connection>;
/// 
/// // Usage example in a system
/// fn my_system(
///     query: Query<&mut Connection>,
/// ) {}
/// ```
pub enum Connection {}

#[derive(QueryData)]
pub struct ConnectionRef<'w> {
    inner: Ref<'w, ConnectionComp>,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct ConnectionMut<'w> {
    inner: Mut<'w, ConnectionComp>,
}

macro_rules! defer_impl {
    (ref $id:ident $tgt:ident) => {
        defer_impl!(raw { &'o $id } $tgt);

        unsafe impl<'o> QueryData for &'o $id {
            type ReadOnly = $tgt<'o>;
        }
    };

    (mut $id:ident $tgt:ident) => {
        defer_impl!(raw { &'o mut $id } $tgt);
    };

    (raw { $id:ty } $tgt:ident) => {
        unsafe impl<'o> WorldQuery for $id {
            type Item<'__k> = <$tgt<'o> as WorldQuery>::Item<'__k>;
            type Fetch<'__k> = <$tgt<'o> as WorldQuery>::Fetch<'__k>;
            type State = <$tgt<'o> as WorldQuery>::State;

            #[inline(always)]
            fn shrink<'wlong: 'wshort, 'wshort>(item: Self::Item<'wlong>) -> Self::Item<'wshort> {
                <$tgt<'o> as WorldQuery>::shrink(
                    item,
                )
            }

            #[inline(always)]
            unsafe fn init_fetch<'w>(
                world: bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell<'w>,
                state: &Self::State,
                last_run: bevy_ecs::component::Tick,
                this_run: bevy_ecs::component::Tick,
            ) -> Self::Fetch<'w> {
                // <EndpointRef<'w> as WorldQuery>::init_fetch(
                //     world,
                //     state,
                //     last_run,
                //     this_run,
                // )

                todo!()
            }

            const IS_DENSE: bool = <$tgt<'o> as WorldQuery>::IS_DENSE;

            #[inline(always)]
            unsafe fn set_archetype<'w>(
                fetch: &mut Self::Fetch<'w>,
                state: &Self::State,
                archetype: &'w bevy_ecs::archetype::Archetype,
                table: &'w bevy_ecs::storage::Table,
            ) {
                // <EndpointRef<'o> as WorldQuery>::set_archetype(
                //     fetch,
                //     state,
                //     archetype,
                //     table,
                // )

                todo!()
            }

            #[inline(always)]
            unsafe fn set_table<'w>(
                fetch: &mut Self::Fetch<'w>,
                state: &Self::State,
                table: &'w bevy_ecs::storage::Table,
            ) {
                // <EndpointRef<'o> as WorldQuery>::set_table(
                //     fetch,
                //     state,
                //     table,
                // )

                todo!()
            }

            #[inline(always)]
            unsafe fn fetch<'w>(
                fetch: &mut Self::Fetch<'w>,
                entity: Entity,
                table_row: bevy_ecs::storage::TableRow,
            ) -> Self::Item<'w> {
                <$tgt<'o> as WorldQuery>::fetch(
                    fetch,
                    entity,
                    table_row,
                )
            }

            #[inline(always)]
            fn update_component_access(
                state: &Self::State,
                access: &mut bevy_ecs::query::FilteredAccess<bevy_ecs::component::ComponentId>,
            ) {
                <$tgt<'o> as WorldQuery>::update_component_access(
                    state,
                    access,
                )
            }

            #[inline(always)]
            fn init_state(
                world: &mut World
            ) -> Self::State {
                <$tgt<'o> as WorldQuery>::init_state(
                    world,
                )
            }

            #[inline(always)]
            fn get_state(
                components: &bevy_ecs::component::Components
            ) -> Option<Self::State> {
                <$tgt<'o> as WorldQuery>::get_state(
                    components,
                )
            }

            #[inline(always)]
            fn matches_component_set(
                state: &Self::State,
                set_contains_id: &impl Fn(bevy_ecs::component::ComponentId) -> bool,
            ) -> bool {
                <$tgt<'o> as WorldQuery>::matches_component_set(
                    state,
                    set_contains_id,
                )
            }
        }    
    };
}

defer_impl!(ref Endpoint EndpointRef);
defer_impl!(mut Endpoint EndpointMut);
defer_impl!(ref Connection ConnectionRef);
defer_impl!(mut Connection ConnectionMut);