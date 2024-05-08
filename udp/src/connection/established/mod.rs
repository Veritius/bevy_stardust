mod polling;
mod systems;
mod writer;

pub(crate) use writer::established_writing_system;
pub(crate) use systems::*;

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

    /// Melt the ice. This puts the peer on 'thinner ice',
    /// which is basically a tracking mechanism that kicks
    /// peers that make too many mistakes while connected.
    pub(crate) fn melt_ice(&mut self, amount: u16) {
        self.ice_thickness = self.ice_thickness.saturating_sub(amount);
    }
}