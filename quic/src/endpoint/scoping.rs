use std::marker::PhantomData;
use bevy::{prelude::*, utils::EntityHashSet, ecs::query::{QueryData, QueryItem, ROQueryItem}};

/// An ID only valid within a scope.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScopedId<'a> {
    id: Entity,
    phantom: PhantomData<&'a ()>,
}

impl<'a> ScopedId<'a> {
    pub(super) unsafe fn new(id: Entity) -> Self {
        Self { id, phantom: PhantomData }
    }

    pub(super) fn inner(&self) -> Entity {
        self.id
    }
}

pub struct Connections<'a> {
    set: &'a EntityHashSet<Entity>,
}

impl<'a> Connections<'a> {
    pub(super) unsafe fn new(set: &'a EntityHashSet<Entity>) -> Self {
        Self { set }
    }

    pub fn iter(&self) -> impl Iterator<Item = ScopedId<'a>> {
        // SAFETY: If the guarantees of `new` are upheld, this is fine.
        self.set.iter().map(|id| unsafe { ScopedId::new(*id) })
    }

    pub fn contains(&self, id: ScopedId<'a>) -> bool {
        self.set.contains(&id.inner())
    }
}

pub struct ScopedAccess<'a, Data: QueryData> {
    query: &'a Query<'a, 'a, Data>,
}

impl<'a, Data: QueryData> ScopedAccess<'a, Data> {
    pub(super) unsafe fn new(query: &'a Query<'a, 'a, Data>) -> Self {
        Self { query }
    }

    pub fn get(&'a self, id: ScopedId<'a>) -> ROQueryItem<'a, Data> {
        // TODO: Safety annotation
        unsafe { self.query.get(id.inner()).unwrap_unchecked() }
    }

    pub fn get_mut(&'a mut self, id: ScopedId<'a>) -> QueryItem<'a, Data> {
        // TODO: Safety annotation
        unsafe { self.query.get_unchecked(id.inner()).unwrap_unchecked() }
    }
}