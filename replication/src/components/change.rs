use std::ops::Deref;
use bevy::{ecs::{component::Tick, query::QueryData, system::SystemChangeTick}, prelude::*};
use crate::change::NetChangeTracking;

/// [`Component`] access with additional change-tracking metadata.
/// 
/// The [`DetectChanges`] implementation is identical to `Ref<R>`.
#[derive(QueryData)]
pub struct NetRef<'w, C: Component> {
    value: Ref<'w, C>,
    netch: &'w NetChangeTracking<C>,
}

impl<'w, C: Component> NetRef<'w, C> {
    /// Returns `true` if and only if the latest change was made by a replication system.
    pub fn is_changed_by_replication(&self, ticks: &SystemChangeTick) -> bool {
        self.netch.cd_inner(
            &self.value,
            ticks.last_run(),
            ticks.this_run(),
            false,
        )
    }

    /// Returns `true` if and only if the latest change was made by the application or another plugin.
    pub fn is_changed_by_application(&self, ticks: &SystemChangeTick) -> bool {
        self.netch.cd_inner(
            &self.value,
            ticks.last_run(),
            ticks.this_run(),
            true,
        )
    }
}

impl<'w, C: Component> DetectChanges for NetRef<'w, C> {
    #[inline]
    fn is_added(&self) -> bool {
        self.value.is_added()
    }

    #[inline]
    fn is_changed(&self) -> bool {
        self.value.is_changed()
    }

    #[inline]
    fn last_changed(&self) -> Tick {
        self.value.last_changed()
    }
}

impl<'w, C: Component> Deref for NetRef<'w, C> {
    type Target = Ref<'w, C>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}