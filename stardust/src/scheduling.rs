//! Schedules used in Stardust.

use bevy::{prelude::*, ecs::{schedule::ScheduleLabel, system::SystemParam}};

/// Information about Stardust's scheduling.
#[derive(SystemParam)]
pub struct NetworkScheduleData<'w> {
    pub(crate) message_mutation_allowed: Res<'w, MessageStorageMutationAllowed>,
}

impl<'w> NetworkScheduleData<'w> {
    /// Returns whether or not mutating `NetworkMessageStorage` components is allowed.
    pub fn message_storage_mutation_allowed(&self) -> bool {
        self.message_mutation_allowed.0
    }
}

// HACK: This prevents adding to `NetworkMessageStorage` components outside of the `TransportReadPackets` schedule.
// TODO: Think of something better than this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub(crate) struct MessageStorageMutationAllowed(pub bool);

pub(super) fn add_schedules(app: &mut App) {
    app.insert_resource(MessageStorageMutationAllowed(false));

    app.add_schedule(Schedule::new(TransportReadPackets));
    app.add_schedule(Schedule::new(PreReadOctetStrings));
    app.add_schedule(Schedule::new(TransportSendPackets));

    app.add_systems(PreUpdate, network_pre_update);
    app.add_systems(PostUpdate, network_post_update);
}

/// Runs during Bevy's PreUpdate and is used for receiving packets from peers and processing them.
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct NetworkPreUpdate;

pub(super) fn network_pre_update(world: &mut World) {
    world.resource_mut::<MessageStorageMutationAllowed>().0 = true;
    world.run_schedule(TransportReadPackets);
    apply_deferred(world);
    world.resource_mut::<MessageStorageMutationAllowed>().0 = false;
    world.run_schedule(PreReadOctetStrings);
    apply_deferred(world);
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
    apply_deferred(world);
}

/// The transport layer fragments and sends packets over the network.
/// 
/// Only transport layer systems should be in here!
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct TransportSendPackets;