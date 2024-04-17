use std::marker::PhantomData;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{prelude::*, serialisation::SerialisationFunctions};

#[derive(Default)]
struct EventReplicationData<T: Event>(PhantomData<T>);

/// An event sent over the network.
#[derive(Event)]
pub struct NetworkEvent<T> {
    /// The peer that sent the event.
    pub origin: Entity,
    /// The event data (`T`)
    pub event: T,
    // Hidden field to prevent manual construction of the struct.
    // Only the plugin should be able to send these events.
    hidden: (),
}

/// Relays the event `T` over the network using the given [`SerialisationFunctions`].
/// It's important that the tick `T` is received does not matter, due to network delay.
/// 
/// Received events will be added as `NetworkEvent<T>`, not `T`,
/// as they contain additional metadata.
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

        app.insert_resource(EventSerialisationFns(self.serialisation.clone()));
        app.add_event::<NetworkEvent<T>>();

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

#[derive(Resource)]
struct EventSerialisationFns<T>(SerialisationFunctions<T>);

/// Only room memberhips
pub struct EventMemberships<T: Event> {
    /// See [`RoomMemberships`].
    pub memberships: RoomMemberships,
    phantom: PhantomData<T>,
}

impl<T: Event> Resource for EventMemberships<T> {}

impl<T: Event> EventMemberships<T> {
    /// Creates a new [`EventMemberships`] resource.
    pub fn new(filter: RoomMemberships) -> Self {
        Self {
            memberships: filter,
            phantom: PhantomData,
        }
    }
}

fn rep_events_receiving_system<T: Event>(
    registry: Res<ChannelRegistry>,
    serialisation: Res<EventSerialisationFns<T>>,
    membership: Res<EventMemberships<T>>,
    mut events: EventWriter<NetworkEvent<T>>,
    peers: Query<(Entity, &NetworkMessages<Incoming>), With<NetworkPeer>>,
) {
    let channel = registry.channel_id(std::any::TypeId::of::<T>()).unwrap();

    // Iterate all peers and read messages for T
    for (peer, messages) in peers.iter() {
        let messages = messages.get(channel);

        // Deserialise takes an owned Bytes, and it's cheap to clone it
        for message in messages.iter().cloned() {
            match (serialisation.0.deserialise)(message) {
                Ok(val) => {
                    events.send(NetworkEvent {
                        origin: peer,
                        event: val,
                        hidden: (),
                    });
                },
                Err(err) => {
                    // TODO: Kick them if they repeatedly send bad packets
                    error!("Error while deserialising event for type {} from {peer:?}: {err}", std::any::type_name::<T>());
                },
            }
        }
    }
}

fn rep_events_sending_system<T: Event>(
    registry: Res<ChannelRegistry>,
    serialisation: Res<EventSerialisationFns<T>>,
    membership: Res<EventMemberships<T>>,
    events: EventReader<T>,
) {
    todo!()
}