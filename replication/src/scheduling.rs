use bevy::prelude::*;
use bevy_stardust::prelude::*;

pub(super) fn setup_schedules(app: &mut App) {
    app.configure_sets(PreUpdate, ReplicationSystems::UpdateEntities
        .before(ReplicationSystems::UpdateComponents)
        .in_set(NetworkRead::Read));

    app.configure_sets(PreUpdate, ReplicationSystems::UpdateEntities
        .in_set(NetworkRead::Read));

    app.configure_sets(PostUpdate, ReplicationSystems::UpdateResources
        .before(NetworkWrite::Send));

    app.configure_sets(PostUpdate, ReplicationSystems::UpdateEntities
        .before(NetworkWrite::Send));

    app.configure_sets(PostUpdate, ReplicationSystems::UpdateComponents
        .before(NetworkWrite::Send));
}

/// Replication system sets.
/// These run in both [`PreUpdate`] and [`PostUpdate`], and your systems should order against them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub enum ReplicationSystems {
    /// Replicate remote resources.
    UpdateResources,

    /// Replicate remote entities.
    UpdateEntities,

    /// Replicate remote components on remote entities.
    /// 
    /// In [`PostUpdate`], this occurs after [`UpdateEntities`](ReplicationSystems::UpdateEntities).
    UpdateComponents,
}