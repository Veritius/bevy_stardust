use std::ops::Deref;
use bevy::{ecs::{query::QueryData, system::SystemChangeTick}, prelude::*};
use crate::change::NetChangeTracking;

/// [`Component`] access with additional change-tracking metadata.
#[derive(QueryData)]
pub struct NetRef<'w, C: Component> {
    value: Ref<'w, C>,
    netch: &'w NetChangeTracking<C>,
}

impl<'w, C: Component> NetRef<'w, C> {
    /// Returns `true` if and only if the latest change was made by a replication system.
    pub fn is_changed_by_replication(&self, ticks: &SystemChangeTick) -> bool {
        (*self.netch).is_changed(
            ticks.last_run(),
            ticks.this_run(),
        )
    }

    /// Returns `true` if and only if the latest change was made by the application or another plugin.
    pub fn is_changed_by_application(&self, ticks: &SystemChangeTick) -> bool {
        self.value.last_changed().is_newer_than(
            self.netch.changed,
            ticks.this_run(),
        )
    }
}

impl<'w, C: Component> Deref for NetRef<'w, C> {
    type Target = Ref<'w, C>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}