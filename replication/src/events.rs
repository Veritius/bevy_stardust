use std::marker::PhantomData;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::prelude::*;

#[derive(Default)]
pub(crate) struct EventReplicationData<T: ReplicableEvent>(PhantomData<T>);

/// Naively relays the event `T` over the network.
/// It's important that the tick `T` is received does not matter.
/// 
/// This plugin must be added before [`StardustPlugin`].
/// Implicitly adds [`ReplicationPlugin`] if not present.
pub struct EventReplicationPlugin<T: ReplicableEvent> {
    /// If replicated events should be sent reliably.
    pub reliability: ReliabilityGuarantee,

    /// If replicated events should be ordered.
    pub ordering: OrderingGuarantee,

    /// The priority of network messages for replicating `T`.
    pub message_priority: u32,

    #[doc(hidden)]
    pub phantom: PhantomData<T>,
}

impl<T: ReplicableEvent> Plugin for EventReplicationPlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<CoreReplicationPlugin>() {
            app.add_plugins(CoreReplicationPlugin);
        }

        app.add_channel::<EventReplicationData<T>>(ChannelConfiguration {
            reliable: self.reliability,
            ordered: self.ordering,
            fragmented: true,
            priority: self.message_priority,
        });

        app.add_systems(PreUpdate, rep_events_receiving_system::<T>
            .in_set(NetworkRead::Read));

        app.add_systems(PostUpdate, rep_events_sending_system::<T>
            .before(NetworkWrite::Send));
    }
}

/// Controls how the event of type `T` is replicated to peers.
pub struct ReplicatedEventRoomMembership<T: ReplicableEvent> {
    /// See [`RoomFilterConfig`].
    pub filter: MembershipFilter,
    phantom: PhantomData<T>,
}

impl<T: ReplicableEvent> Resource for ReplicatedEventRoomMembership<T> {}

impl<T: ReplicableEvent> ReplicatedEventRoomMembership<T> {
    /// Creates a new [`ReplicatedEventRoomMembership`] resource.
    pub fn new(filter: MembershipFilter) -> Self {
        Self {
            filter,
            phantom: PhantomData,
        }
    }
}

fn rep_events_receiving_system<T: ReplicableEvent>(
    registry: Res<ChannelRegistry>,
    membership: Res<ReplicatedEventRoomMembership<T>>,
    mut events: EventWriter<T>,
) {
    todo!()
}

fn rep_events_sending_system<T: ReplicableEvent>(
    registry: Res<ChannelRegistry>,
    membership: Res<ReplicatedEventRoomMembership<T>>,
    events: EventReader<T>,
) {
    todo!()
}