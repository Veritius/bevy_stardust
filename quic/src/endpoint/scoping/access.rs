use bevy::{ecs::query::{QueryData, QueryItem, ROQueryItem}, prelude::*};
use super::id::ScopedId;

pub struct ScopedAccess<'a, Data: QueryData> {
    query: &'a Query<'a, 'a, Data>,
}

impl<'a, Data: QueryData> ScopedAccess<'a, Data> {
    pub(super) unsafe fn new(query: &'a Query<'a, 'a, Data>) -> Self {
        Self { query}
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