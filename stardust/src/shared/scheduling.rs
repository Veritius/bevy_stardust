use bevy::{ecs::schedule::ScheduleLabel, prelude::World};

/// Runs during Bevy's `PreUpdate` and contains [ReadPackets], [ProcessMessages], and [NetworkPreUpdateCleanup].
/// 
/// Used for receiving packets from peers and processing them.
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct NetworkPreUpdate;

pub(super) fn network_pre_update(world: &mut World) {
    world.run_schedule(ReadPackets);
    world.run_schedule(ProcessMessages);
    world.run_schedule(NetworkPreUpdateCleanup);
}

/// Receive packets from UDP sockets and process them to yield usable payloads. Runs during [NetworkPreUpdate].
/// Only visible inside the Stardust crate.
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub(crate) struct ReadPackets;

/// Read bytes and turn into events, etc. Runs during [NetworkPreUpdate].
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct ProcessMessages;

/// Clean up anything unnecessary. Runs during [NetworkPreUpdate].
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct NetworkPreUpdateCleanup;

/// Runs during Bevy's `PostUpdate` and contains [SendPackets]
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct NetworkPostUpdate;

pub(super) fn network_post_update(world: &mut World) {
    world.run_schedule(SendPackets);
}

/// Sends packets to peers. Runs during [NetworkPostUpdate].
/// Only visible inside the Stardust crate.
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScheduleLabel)]
pub(crate) struct SendPackets;