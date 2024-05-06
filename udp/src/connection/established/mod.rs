mod systems;
mod reader;
mod writer;

use std::time::Duration;
use bevy::prelude::*;
use bevy_stardust::prelude::*;

use super::{ordering::OrderingManager, packets::{builder::PacketBuilder, frames::SendFrame, reader::PacketReader}, reliability::{ReliabilityState, ReliablePackets}};
pub(crate) use reader::established_packet_reader_system;
pub(crate) use writer::established_packet_writing_system;
pub(crate) use systems::established_timeout_system;

#[derive(Component)]
pub(crate) struct Established {
    reliable_timeout: Duration,
    reliability: ReliablePackets,
    orderings: OrderingManager,
    errors: u32,

    reader: PacketReader,
    builder: PacketBuilder,
}

impl Established {
    pub(in super::super) fn new(
        reliability: &ReliabilityState,
        registry: &ChannelRegistryInner,
    ) -> Self {
        Self {
            reliable_timeout: Duration::from_millis(1000), // TODO: Make this a dynamic value based off RTT
            reliability: ReliablePackets::new(reliability.clone()),
            orderings: OrderingManager::new(),
            errors: 0,

            reader: PacketReader::default(),
            builder: PacketBuilder::default(),
        }
    }

    pub(super) fn queue_send_frame(&mut self, frame: SendFrame) {
        self.builder.put(frame);
    }
}