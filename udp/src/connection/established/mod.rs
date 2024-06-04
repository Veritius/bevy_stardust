mod closing;
mod control;
mod frames;
mod polling;
mod writer;

pub(super) use polling::established_reading_system;
pub(super) use writer::established_writing_system;
pub(super) use closing::{
    established_close_events_system,
    established_close_frames_system,
    established_close_despawn_system,
};

use bevy::prelude::*;
use super::{ordering::OrderingManager, reliability::{ReliabilityState, ReliablePackets}};
use frames::{builder::PacketBuilder, reader::PacketParser};

#[derive(Component)]
pub(crate) struct Established {
    reliability: ReliablePackets,
    orderings: OrderingManager,

    reader: PacketParser,
    builder: PacketBuilder,

    closing: bool,
    ice_thickness: u16,
}

impl Established {
    pub(in super::super) fn new(
        reliability: ReliabilityState,
    ) -> Self {
        Self {
            reliability: ReliablePackets::new(reliability.clone()),
            orderings: OrderingManager::new(),

            reader: PacketParser::default(),
            builder: PacketBuilder::default(),

            closing: false,
            ice_thickness: u16::MAX,
        }
    }
}