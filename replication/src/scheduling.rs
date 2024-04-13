use bevy::prelude::*;

pub(super) fn setup_schedules(app: &mut App) {

}

/// System sets run in [`PreUpdate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub enum PreUpdateReplicationSystems {

}

/// System sets run in [`PostUpdate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub enum PostUpdateReplicationSystems {

}