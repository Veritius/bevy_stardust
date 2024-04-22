use std::ops::Deref;
use bevy::{ecs::system::{SystemChangeTick, SystemParam}, prelude::*};
use crate::change::NetChangeTracking;

/// [`Resource`] access with additional change-tracking metadata.
#[derive(SystemParam)]
pub struct NetRes<'w, R: Resource> {
    value: Res<'w, R>,
    netch: Res<'w, NetChangeTracking<R>>,
    ticks: SystemChangeTick,
}

impl<'w, R: Resource> NetRes<'w, R> {
    /// Returns `true` if and only if the latest change was made by a replication system.
    pub fn is_changed_by_replication(&self) -> bool {
        todo!()
    }

    /// Returns `true` if and only if the latest change was made by a game system.
    pub fn is_changed_by_application(&self) -> bool {
        self.value.last_changed().is_newer_than(
            self.netch.changed,
            self.ticks.this_run()
        )
    }
}

impl<'w, R: Resource> Deref for NetRes<'w, R> {
    type Target = Res<'w, R>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}