use std::marker::PhantomData;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{prelude::*, serialisation::SerialisationFunctions};

/// Relays the event `T` over the network using the given [`SerialisationFunctions`].
/// 
/// Events received over the network are not added as type `T`.
/// They are instead added as a new event type, [`RelayEvent<T>`].
pub struct EventRelayPlugin<T: Event> {
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

impl<T: Event> Plugin for EventRelayPlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<CoreReplicationPlugin>() {
            app.add_plugins(CoreReplicationPlugin);
        }

        app.add_event::<T>();
        app.add_event::<RelayedEvent<T>>();

        app.insert_resource(EventSerialisationFns(self.serialisation.clone()));

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

#[derive(Default)]
struct EventReplicationData<T: Event>(PhantomData<T>);

#[derive(Resource)]
struct EventSerialisationFns<T>(SerialisationFunctions<T>);

/// An event received by a remote peer.
#[derive(Event)]
pub struct RelayedEvent<T> {
    /// The peer that sent the event.
    pub origin: Entity,
    /// The event data (`T`)
    pub event: T,
    // Hidden field to prevent manual construction of the struct.
    // Only the plugin should be able to send these events.
    _hidden: (),
}

/// Convenience type, wrapping `EventReader<RelayedEvent<T>>`.
pub type NetEventReader<'w, 's, T> = EventReader<'w, 's, RelayedEvent<T>>;

fn rep_events_receiving_system<T: Event>(
    registry: Res<ChannelRegistry>,
    serialisation: Res<EventSerialisationFns<T>>,
    mut events: EventWriter<RelayedEvent<T>>,
    peers: Query<(Entity, &NetworkMessages<Incoming>), (With<NetworkPeer>, With<ReplicationPeer>)>,
) {
    // Avoid wasting our time
    if peers.is_empty() { return; }

    let channel = registry.channel_id(std::any::TypeId::of::<EventReplicationData<T>>()).unwrap();
    let type_name = std::any::type_name::<T>();

    // Iterate all peers and read messages for T
    for (peer, messages) in peers.iter() {
        let messages = messages.get(channel);

        // Deserialise takes an owned Bytes, and it's cheap to clone it
        for message in messages.iter().cloned() {
            match (serialisation.0.deserialise)(message) {
                Ok(val) => {
                    events.send(RelayedEvent {
                        origin: peer,
                        event: val,
                        _hidden: (),
                    });
                },
                Err(err) => {
                    // TODO: Kick them if they repeatedly send bad packets
                    error!("Error while deserialising event {type_name} from {peer:?}: {err}");
                },
            }
        }
    }
}

fn rep_events_sending_system<T: Event>(
    registry: Res<ChannelRegistry>,
    serialisation: Res<EventSerialisationFns<T>>,
    mut events: EventReader<T>,
    mut peers: Query<&mut NetworkMessages<Outgoing>, (With<NetworkPeer>, With<ReplicationPeer>)>,
) {
    // Avoid wasting our time
    if events.is_empty() || peers.is_empty() { return; }

    let channel = registry.channel_id(std::any::TypeId::of::<EventReplicationData<T>>()).unwrap();
    let type_name = std::any::type_name::<T>();

    // Serialise everything ahead of time, since it's expensive
    // and cloning Bytes objects is a very cheap thing to do
    let serialised = events
    .read()
    .filter_map(|event| {
        match (serialisation.0.serialise)(event) {
            Ok(bytes) => Some(bytes),
            Err(err) => {
                error!("Error while serialising event {}: {err}", type_name);
                return None;
            },
        }
    })
    .collect::<Vec<_>>();

    // Add serialised messages to all peers' message queues
    for mut messages in peers.iter_mut() {
        for bytes in serialised.iter().cloned() {
            messages.push(channel, bytes);
        }
    }
}