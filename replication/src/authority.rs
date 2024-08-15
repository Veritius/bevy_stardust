//! Authority configuration.

use bevy::{ecs::query::{QueryData, QueryFilter, WorldQuery}, prelude::*};

#[derive(Component)]
pub(crate) struct AuthorityData {
    id: AuthorityId,
}

impl AuthorityData {
    pub fn new(authority: AuthorityId) -> Self {
        Self {
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthorityId {
    Local,
    Remote(Entity)
}

/// A [`QueryFilter`] for entities this peer holds authority over.
#[derive(QueryData)]
pub struct Controlling<'a> {
    authority: &'a AuthorityData,
}

impl<'a> QueryFilter for Controlling<'a> {
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

/// A [`QueryFilter`] for entities that another peer has authority over.
#[derive(QueryData)]
pub struct Controlled<'a> {
    authority: &'a AuthorityData,
}

impl<'a> QueryFilter for Controlled<'a> {
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