use std::marker::PhantomData;
use bevy::{ecs::component::*, prelude::*};

pub(crate) struct NetChangeTracking<T> {
    changed: Tick,
    phantom: PhantomData<T>,
}

impl<T> NetChangeTracking<T> {
    #[inline]
    pub fn is_changed(&self, last_run: Tick, this_run: Tick) -> bool {
        self.changed.is_newer_than(last_run, this_run)
    }

    #[inline]
    pub fn last_changed(&self) -> Tick {
        self.changed
    }

    #[inline]
    pub fn set_change_tick(&mut self, tick: Tick) {
        self.changed = tick;
    }

    pub fn cd_inner<C: DetectChanges>(&self, last_run: Tick, this_run: Tick, other: &C, inv: bool) -> bool {
        // Check if it's been changed at all
        if !other.last_changed().is_newer_than(last_run, this_run) {
            return false;
        }

        // Check if the change was from a replication system
        if self.last_changed() == other.last_changed() {
            return match inv {
                false => true,
                true => false
            };
        }

        // Otherwise, return false
        return false;
    }
}

impl<T: Component> Component for NetChangeTracking<T> {
    type Storage = TableStorage;
}

impl<T: Resource> Resource for NetChangeTracking<T> {}