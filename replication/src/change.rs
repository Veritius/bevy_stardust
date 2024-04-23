//! Types for special tracking in change detection.

use std::marker::PhantomData;
use bevy::{ecs::component::*, prelude::*};

/// Change tracking for the replicated component/resource `T`.
/// Currently must be added manually.
/// This will change in future, and this type will be made private.
/// Don't rely on it.
pub struct NetChangeTracking<T> {
    changed: Tick,
    phantom: PhantomData<T>,
}

impl<T> NetChangeTracking<T> {
    /// Returns whether or not this has been changed **by a replication system** since this system last ran.
    #[inline]
    pub fn is_changed(&self, last_run: Tick, this_run: Tick) -> bool {
        self.changed.is_newer_than(last_run, this_run)
    }

    /// Returns the last time a replication system changed `R`.
    #[inline]
    pub fn last_changed(&self) -> Tick {
        self.changed
    }

    #[inline]
    pub(crate) fn set_change_tick(&mut self, tick: Tick) {
        self.changed = tick;
    }

    pub(crate) fn cd_inner<C: DetectChanges>(&self, last_run: Tick, this_run: Tick, other: &C, inv: bool) -> bool {
        let has_changed = other.last_changed().is_newer_than(last_run, this_run);
        let was_repl_change = self.last_changed() == other.last_changed();

        match (has_changed, was_repl_change, inv) {
            (true, true, false) => true,
            (true, true, true) => false,
            _ => false,
        }
    }
}

// TODO(Bevy 0.14): Use component/resource hooks.
impl<T> FromWorld for NetChangeTracking<T> {
    fn from_world(world: &mut World) -> Self {
        Self {
            changed: world.change_tick(),
            phantom: PhantomData,
        }
    }
}

impl<C: Component> Component for NetChangeTracking<C> {
    type Storage = TableStorage;
}

impl<R: Resource> Resource for NetChangeTracking<R> {}