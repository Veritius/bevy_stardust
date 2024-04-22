use std::marker::PhantomData;
use bevy::{ecs::component::*, prelude::*};

pub(crate) struct NetChangeTracking<T> {
    pub changed: Tick,
    phantom: PhantomData<T>,
}

impl<T> NetChangeTracking<T> {
    #[inline]
    pub fn is_changed(&self, last_run: Tick, this_run: Tick) -> bool {
        self.changed.is_newer_than(last_run, this_run)
    }
}

impl<T: Component> Component for NetChangeTracking<T> {
    type Storage = TableStorage;
}

impl<T: Resource> Resource for NetChangeTracking<T> {}