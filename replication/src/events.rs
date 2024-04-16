use std::marker::PhantomData;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::prelude::*;

#[derive(Default)]
pub(crate) struct EventReplicationData<T: ReplicableEvent>(PhantomData<T>);

/// Naively relays the event `T` over the network.
/// 
/// This plugin must be added before [`StardustPlugin`].
/// Implicitly adds [`ReplicationPlugin`] if not present.
pub struct ReplicateEventsPlugin<T: ReplicableEvent> {
    /// If replication data should be sent reliably.
    pub reliability: ReliabilityGuarantee,

    /// Sets how events are ordered.
    pub ordering: OrderingGuarantee,

    /// The priority of the resource to replicate.
    /// Higher priority items will be replicated first.
    pub priority: u32,

    #[doc(hidden)]
    pub phantom: PhantomData<T>,
}

impl<T: ReplicableEvent> Plugin for ReplicateEventsPlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<CoreReplicationPlugin>() {
            app.add_plugins(CoreReplicationPlugin);
        }

        app.add_channel::<EventReplicationData<T>>(ChannelConfiguration {
            reliable: self.reliability,
            ordered: self.ordering,
            fragmented: true,
            priority: self.priority,
        });
    }
}