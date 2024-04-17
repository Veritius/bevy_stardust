use std::{marker::PhantomData, sync::Arc};
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{prelude::*, serialisation::SerialisationFunctions};

#[derive(Default)]
pub(crate) struct EventReplicationData<T: Event>(PhantomData<T>);

/// Naively relays the event `T` over the network using the given [`SerialisationFunctions`].
/// It's important that the tick `T` is received does not matter, due to network delay.
pub struct EventReplicationPlugin<T: Event, ReadIn = (), SendIn = ()> {
    /// Read condition for receiving replicated events of type `T`.
    pub read_condition: Option<Arc<dyn ReadOnlySystem<In = ReadIn, Out = bool>>>,

    /// Send condition for sending replicated events of type `T`.
    pub send_condition: Option<Arc<dyn ReadOnlySystem<In = SendIn, Out = bool>>>,

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

// A hack to work around not being able to 'ignore' the ReadIn and SendIn arguments.
// This is necessary to avoid more than one EventReplicationPlugin being added for T.
// This can be removed when there's a macro to concatenate literals with the
// output from std::any::type_name. Until then...
#[derive(Resource)]
struct PluginAdded<T>(PhantomData<T>);

impl<T: Event> Plugin for EventReplicationPlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<CoreReplicationPlugin>() {
            app.add_plugins(CoreReplicationPlugin);
        }

        // Check if the plugin was already added
        if app.world.contains_resource::<PluginAdded<T>>() {
            panic!("EventReplicationPlugin<{}> was already added to the App", std::any::type_name::<T>());
        }

        // Mark the plugin as added
        app.insert_resource(PluginAdded(PhantomData::<T>));

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

    fn finish(&self, app: &mut App) {
        // Remove the marker resource, since it's no longer needed
        app.world.remove_resource::<PluginAdded<T>>();
    }
}

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
    membership: Res<EventMemberships<T>>,
    mut events: EventWriter<T>,
) {
    todo!()
}

fn rep_events_sending_system<T: Event>(
    registry: Res<ChannelRegistry>,
    membership: Res<EventMemberships<T>>,
    events: EventReader<T>,
) {
    todo!()
}