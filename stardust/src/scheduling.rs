//! Schedules used in Stardust.

use bevy::{prelude::*, ecs::{schedule::ScheduleLabel, system::SystemParam}};

/// Data about the network schedule.
#[derive(SystemParam)]
pub struct NetworkScheduleData<'w> {
    pub(crate) message_mutation_allowed: Res<'w, MessageStorageMutationAllowed>,
}

// HACK: This prevents adding to `NetworkMessageStorage` components outside of the `TransportReadPackets` schedule.
// TODO: Think of something better than this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub(crate) struct MessageStorageMutationAllowed(pub bool);

pub(super) fn add_schedules(app: &mut App) {
    app.insert_resource(MessageStorageMutationAllowed(false));

    app.add_schedule(TransportReadPackets, Schedule::new());
    app.add_schedule(PreReadOctetStrings, Schedule::new());
    app.add_schedule(TransportSendPackets, Schedule::new());

    app.add_systems(PreUpdate, network_pre_update);
    app.add_systems(PostUpdate, network_post_update);
}

/// Runs during Bevy's PreUpdate and is used for receiving packets from peers and processing them.
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct NetworkPreUpdate;

pub(super) fn network_pre_update(world: &mut World) {
    world.resource_mut::<MessageStorageMutationAllowed>().0 = true;
    world.run_schedule(TransportReadPackets);
    world.resource_mut::<MessageStorageMutationAllowed>().0 = false;
    world.run_schedule(PreReadOctetStrings);
}

/// Receive packets and process them into usable data (ordering, defragmentation)
///
/// Only transport layer systems should be in here!
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct TransportReadPackets;

/// Read bytes before the main `Update` schedule.
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct PreReadOctetStrings;

/// Runs during Bevy's `PostUpdate` and deals with **sending data.**
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct NetworkPostUpdate;

pub(super) fn network_post_update(world: &mut World) {
    world.run_schedule(TransportSendPackets);
}

/// The transport layer fragments and sends packets over the network.
/// 
/// Only transport layer systems should be in here!
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct TransportSendPackets;