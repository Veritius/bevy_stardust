mod closing;
mod control;
mod polling;
mod writer;

pub(crate) use polling::established_polling_system;
pub(crate) use writer::established_writing_system;

use bevy::prelude::*;
use super::{ordering::OrderingManager, packets::{builder::PacketBuilder, reader::PacketReader}, reliability::{ReliabilityState, ReliablePackets}};

#[derive(Component)]
pub(crate) struct Established {
    reliability: ReliablePackets,
    orderings: OrderingManager,

    reader: PacketReader,
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

            reader: PacketReader::default(),
            builder: PacketBuilder::default(),

            ice_thickness: u16::MAX,
        }
    }
}