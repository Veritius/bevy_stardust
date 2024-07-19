use std::marker::PhantomData;
use bevy::{ecs::query::{QueryData, QueryItem, ROQueryItem}, prelude::*};

/// An ID only valid within a scope.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScopedId<'a> {
    id: Entity,
    phantom: PhantomData<&'a ()>,
}

impl<'a> ScopedId<'a> {
    pub(super) unsafe fn new(id: Entity) -> Self {
        Self {
            id,
            phantom: PhantomData,
        }
    }

    pub(super) fn inner(&self) -> Entity {
        self.id
    }
}

pub struct ScopedAccess<'a, Data: QueryData> {
    pub(super) query: &'a Query<'a, 'a, Data>,
}

impl<'a, Data: QueryData> ScopedAccess<'a, Data> {
    pub fn get(&'a self, id: ScopedId<'a>) -> ROQueryItem<'a, Data> {
        // TODO: Safety annotation
        unsafe { self.query.get(id.inner()).unwrap_unchecked() }
    }

    pub fn get_mut(&'a mut self, id: ScopedId<'a>) -> QueryItem<'a, Data> {
        // TODO: Safety annotation
        unsafe { self.query.get_unchecked(id.inner()).unwrap_unchecked() }
    }
}