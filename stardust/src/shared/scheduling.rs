use bevy::{ecs::schedule::ScheduleLabel, prelude::{World, Resource}};

/// Runs during Bevy's PreUpdate and is used for receiving packets from peers and processing them.
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct NetworkPreUpdate;

pub(super) fn network_pre_update(world: &mut World) {
    world.run_schedule(TransportReadPackets);
    world.run_schedule(ReadOctetStrings);
    world.run_schedule(NetworkPreUpdateCleanup);
}

/// Receive packets and process them into usable data (ordering, defragmentation)
///
/// Only transport layer systems should be in here!
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct TransportReadPackets;

/// Read bytes and turn into events, etc.
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct ReadOctetStrings;

/// Clean up anything unnecessary.
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct NetworkPreUpdateCleanup;

/// Runs during Bevy's `PostUpdate` and deals with **sending data.**
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct NetworkPostUpdate;

pub(super) fn network_post_update(world: &mut World) {
    world.run_schedule(WriteOctetStrings);
    
    world.insert_resource(IsTransportSendPackets);
    world.run_schedule(TransportSendPackets);
    world.remove_resource::<IsTransportSendPackets>();
    
    world.run_schedule(NetworkPostUpdateCleanup);
}

#[derive(Resource)]
pub(super) struct IsTransportSendPackets;

/// Bevy systems write octet strings to be sent over the network.
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct WriteOctetStrings;

/// The transport layer fragments and sends packets over the network.
/// 
/// Only transport layer systems should be in here!
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct TransportSendPackets;

/// Clean up anything unnecessary.
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct NetworkPostUpdateCleanup;