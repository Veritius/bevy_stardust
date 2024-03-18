mod frame;
mod systems;

use std::collections::HashMap;
use bevy_stardust::channels::ChannelId;
use bevy_ecs::prelude::*;
use self::frame::PacketFrame;
use super::{ordering::OrderedMessages, reliability::{ReliabilityState, ReliablePackets}};
pub(crate) use systems::{
    established_packet_reader_system,
    established_packet_builder_system,
};


#[derive(Component)]
pub(crate) struct Established {
    frames: Vec<PacketFrame>,
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
            reliability: ReliablePackets::new(reliability.clone()),
            ordering: HashMap::default(),
        }
    }

    pub(super) fn ordering(&mut self, channel: ChannelId) -> &mut OrderedMessages {
        self.ordering.entry(channel)
            .or_insert(OrderedMessages::new())
    }
}