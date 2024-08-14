//! Change detection for replicated objects.

use bevy::ecs::component::Tick;

/// Change detection state.
#[derive(Debug, Clone)]
pub struct ReplicationTicks {
    local: Option<Tick>,
    remote: Option<Tick>,
}

impl ReplicationTicks {
    /// Returns `true` if the component has changed, and the change was done by this application.
    pub fn is_changed_locally(&self, last_run: Tick, this_run: Tick) -> bool {
        if self.local.is_none() { return false }
        self.local.unwrap().is_newer_than(last_run, this_run)
    }

    /// Returns `true` if the component has changed, and the change was done by a remote application.
    pub fn is_changed_remotely(&self, last_run: Tick, this_run: Tick) -> bool {
        if self.remote.is_none() { return false }
        self.remote.unwrap().is_newer_than(last_run, this_run)
    }

    /// The last time the value was changed by this application.
    /// Returns `None` if the value has never been changed locally.
    pub fn last_changed_locally(&self) -> Option<Tick> {
        self.local
    }

    /// The last time the value was changed by a remote application.
    /// Returns `None` if the value has never been changed remotely.
    pub fn last_changed_remotely(&self) -> Option<Tick> {
        self.remote
    }
}