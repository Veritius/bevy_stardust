mod packing;
mod frame;
mod systems;

use std::collections::HashMap;

use bevy_stardust::channels::ChannelId;
use bytes::Bytes;
use bevy_ecs::prelude::*;
use self::{frame::PacketFrame, packing::PackingManager};
use super::{ordering::OrderedMessages, reliability::{ReliabilityState, ReliablePackets}};
pub(crate) use systems::{
    established_packet_reader_system,
    established_packet_builder_system,
};


#[derive(Component)]
pub(crate) struct Established {
    frames: Vec<PacketFrame>,
    reliable_packer: PackingManager,
    unreliable_packer: PackingManager,
    reliability: ReliablePackets,
    ordering: HashMap<ChannelId, OrderedMessages>,
}

impl Established {
    pub(in super::super) fn new(
        packet_size: usize,
        reliability: &ReliabilityState,
    ) -> Self {
        Self {
            frames: Vec::with_capacity(8),
            reliable_packer: PackingManager::new(packet_size),
            unreliable_packer: PackingManager::new(packet_size),
            reliability: ReliablePackets::new(reliability.clone()),
            ordering: HashMap::default(),
        }
    }

    pub(super) fn ordering(&mut self, channel: ChannelId) -> &mut OrderedMessages {
        self.ordering.entry(channel)
            .or_insert(OrderedMessages::new())
    }
}