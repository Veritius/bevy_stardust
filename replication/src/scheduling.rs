use bevy::prelude::*;
use bevy_stardust::prelude::*;

pub(super) fn setup_schedules(app: &mut App) {
    app.configure_sets(PostUpdate, PostUpdateReplicationSystems::DetectChanges
        .before(PostUpdateReplicationSystems::SendMessages));

    app.configure_sets(PostUpdate, PostUpdateReplicationSystems::SendMessages
        .before(NetworkWrite::Send));
}

/// System sets run in [`PreUpdate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub(crate) enum PreUpdateReplicationSystems {

}

/// System sets run in [`PostUpdate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub(crate) enum PostUpdateReplicationSystems {
    /// Detect changes in the world before serialisation operations.
    /// If you're making changes to replicated data, order your systems to run before this point.
    DetectChanges,

    /// Queues messages for sending.
    SendMessages,
}