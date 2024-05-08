mod control;
mod polling;
mod systems;
mod writer;

pub(crate) use polling::established_polling_system;
pub(crate) use writer::established_writing_system;
pub(crate) use systems::established_timeout_system;

use bevy::prelude::*;
use self::control::Controller;
use super::{ordering::OrderingManager, packets::{builder::PacketBuilder, reader::PacketReader}, reliability::{ReliabilityState, ReliablePackets}};

#[derive(Component)]
pub(crate) struct Established {
    controller: Controller,

    reliability: ReliablePackets,
    orderings: OrderingManager,

    reader: PacketReader,
    builder: PacketBuilder,
}

impl Established {
    pub(in super::super) fn new(
        reliability: &ReliabilityState,
    ) -> Self {
        Self {
            controller: Controller::default(),

            reliability: ReliablePackets::new(reliability.clone()),
            orderings: OrderingManager::new(),

            reader: PacketReader::default(),
            builder: PacketBuilder::default(),
        }
    }
}