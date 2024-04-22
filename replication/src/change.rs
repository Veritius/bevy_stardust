use std::marker::PhantomData;
use bevy::prelude::*;

/// Change tracking for network replicated types.
/// 
/// Currently, change detection is not automatic.
/// You must mark components as 'dirtied' with [`dirty`](Self::dirty).
/// This will change in future, and the `dirty` method will be deprecated.
#[derive(Debug, Reflect)]
#[reflect(Debug, where T: std::fmt::Debug)]
pub struct NetChanges<T> {
    dirty: bool,
    phantom: PhantomData<T>,
}

impl<T: Resource> Resource for NetChanges<T> {}

impl<T: Component> Component for NetChanges<T> {
    type Storage = T::Storage;
}

impl<T> NetChanges<T> {
    /// "Dirties" the component, marking it to be updated on peers.
    #[inline]
    pub fn dirty(&mut self) {
        self.dirty = true;
    }

    /// Returns `true` if the component has been dirtied this tick.
    #[inline]
    pub fn is_dirtied(&self) -> bool {
        self.dirty
    }
}

pub(crate) fn undirty_components_system<T: Component>(
    mut query: Query<&mut NetChanges<T>>,
) {
    query.iter_mut().for_each(|mut v| v.dirty = false);
}

pub(crate) fn undirty_resource_system<T: Resource>(
    mut res: ResMut<NetChanges<T>>,
) {
    res.dirty = false;
}