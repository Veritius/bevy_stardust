use std::marker::PhantomData;
use bevy::{prelude::*, utils::EntityHashSet};

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

pub struct Connections<'a>(&'a EntityHashSet<Entity>);

impl<'a> Connections<'a> {
    pub(super) unsafe fn new(set: &'a EntityHashSet<Entity>) -> Self {
        Self(set)
    }

    pub fn iter(&self) -> impl Iterator<Item = ScopedId<'a>> {
        // SAFETY: If the guarantees of `new` are upheld, this is fine.
        self.0.iter().map(|id| unsafe { ScopedId::new(*id) })
    }

    pub fn contains(&self, id: ScopedId<'a>) -> bool {
        self.0.contains(&id.inner())
    }
}