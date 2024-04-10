use bevy_stardust::prelude::*;
use crate::Replicable;

#[derive(Default)]
#[cfg_attr(feature="reflect", derive(bevy::reflect::Reflect), reflect(from_reflect = false))]
pub(crate) struct ReplicationData<T: Replicable>(T);

/// Messaging configuration for channels used for replication.
pub struct ReplicationChannelConfiguration {
    /// See [`ReliabilityGuarantee`].
    pub reliable: ReliabilityGuarantee,
    /// See the [`priority`](ChannelConfiguration::priority) field in [`ChannelConfiguration`].
    pub priority: u32,
}