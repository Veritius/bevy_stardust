use std::marker::PhantomData;
use bevy::{ecs::{component::StorageType, query::{QueryData, QueryFilter, WorldQuery}}, prelude::*};

pub(crate) struct AuthorityData<T = Entity>
where
    T: Send + Sync + 'static,
{
    _ph: PhantomData<T>,

    id: AuthorityId,
}

impl<T> AuthorityData<T>
where
    T: Send + Sync + 'static,
{
    pub fn new(authority: AuthorityId) -> Self {
        Self {
            _ph: PhantomData,

            id: authority,
        }
    }

    pub fn set_local_auth(&mut self) {
        self.id = AuthorityId::Local;
    }

    pub fn set_remote_auth(&mut self, id: Entity) {
        self.id = AuthorityId::Remote(id);
    }
}

impl<T> Component for AuthorityData<T>
where
    T: Send + Sync + 'static,
{
    const STORAGE_TYPE: StorageType = StorageType::Table;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthorityId {
    Local,
    Remote(Entity)
}

/// A [`QueryFilter`] for entities and components that we have authority over.
#[derive(QueryData)]
pub struct Controller<'a, T = Entity>
where
    T: Send + Sync + 'static,
{
    authority: &'a AuthorityData<T>,
}

impl<'a, T> QueryFilter for Controller<'a, T>
where
    T: Send + Sync + 'static,
{
    const IS_ARCHETYPAL: bool = false;

    #[inline]
    unsafe fn filter_fetch(
        fetch: &mut Self::Fetch<'_>,
        entity: Entity,
        table_row: bevy::ecs::storage::TableRow,
    ) -> bool {
        // SAFETY: The invariants are upheld by the caller
        let f = unsafe { Self::fetch(fetch, entity, table_row) };
        return f.authority.id == AuthorityId::Local;
    }
}

/// A [`QueryFilter`] for entities and components that another peer has authority over.
#[derive(QueryData)]
pub struct Controlled<'a, T = Entity>
where
    T: Send + Sync + 'static,
{
    authority: &'a AuthorityData<T>,
}

impl<'a, T> QueryFilter for Controlled<'a, T>
where
    T: Send + Sync + 'static,
{
    const IS_ARCHETYPAL: bool = false;

    #[inline]
    unsafe fn filter_fetch(
        fetch: &mut Self::Fetch<'_>,
        entity: Entity,
        table_row: bevy::ecs::storage::TableRow,
    ) -> bool {
        // SAFETY: The invariants are upheld by the caller
        let f = unsafe { Self::fetch(fetch, entity, table_row) };
        return f.authority.id != AuthorityId::Local;
    }
}