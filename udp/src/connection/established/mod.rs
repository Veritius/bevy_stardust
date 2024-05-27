mod control;
mod frames;
mod polling;
mod writer;

pub(crate) use polling::established_polling_system;
pub(crate) use writer::established_writing_system;

use bevy::prelude::*;
use super::{ordering::OrderingManager, reliability::{ReliabilityState, ReliablePackets}};
use frames::{builder::PacketBuilder, reader::PacketParser};

#[derive(Component)]
pub(crate) struct Established {
    reliability: ReliablePackets,
    orderings: OrderingManager,

    reader: PacketParser,
    builder: PacketBuilder,

    ice_thickness: u16,
}

impl Established {
    pub(in super::super) fn new(
        reliability: &ReliabilityState,
    ) -> Self {
        Self {
            reliability: ReliablePackets::new(reliability.clone()),
            orderings: OrderingManager::new(),

            reader: PacketParser::default(),
            builder: PacketBuilder::default(),

            ice_thickness: u16::MAX,
        }
    }
}