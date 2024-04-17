use std::marker::PhantomData;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{prelude::*, serialisation::SerialisationFunctions};

#[derive(Default)]
pub(crate) struct EventReplicationData<T: Event>(PhantomData<T>);

/// Naively relays the event `T` over the network.
/// It's important that the tick `T` is received does not matter.
/// 
/// This plugin must be added before [`StardustPlugin`].
/// Implicitly adds [`ReplicationPlugin`] if not present.
pub struct EventReplicationPlugin<T: Event> {
    /// Functions used to serialise and deserialize `T`.
    /// See the [`SerialisationFunctions`] documentation for more information.
    pub serialisation: SerialisationFunctions<T>,

    /// If replicated events should be sent reliably.
    pub reliability: ReliabilityGuarantee,

    /// If replicated events should be ordered.
    pub ordering: OrderingGuarantee,

    /// The priority of network messages for replicating `T`.
    pub message_priority: u32,

    #[doc(hidden)]
    pub phantom: PhantomData<T>,
}

impl<T: Event> Plugin for EventReplicationPlugin<T> {
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
pub struct ReplicatedEventRoomMembership<T: Event> {
    /// See [`RoomFilterConfig`].
    pub filter: RoomMemberships,
    phantom: PhantomData<T>,
}

impl<T: Event> Resource for ReplicatedEventRoomMembership<T> {}

impl<T: Event> ReplicatedEventRoomMembership<T> {
    /// Creates a new [`ReplicatedEventRoomMembership`] resource.
    pub fn new(filter: RoomMemberships) -> Self {
        Self {
            filter,
            phantom: PhantomData,
        }
    }
}

fn rep_events_receiving_system<T: Event>(
    registry: Res<ChannelRegistry>,
    membership: Res<ReplicatedEventRoomMembership<T>>,
    mut events: EventWriter<T>,
) {
    todo!()
}

fn rep_events_sending_system<T: Event>(
    registry: Res<ChannelRegistry>,
    membership: Res<ReplicatedEventRoomMembership<T>>,
    events: EventReader<T>,
) {
    todo!()
}