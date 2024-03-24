mod frame;
mod packing;
mod systems;

use std::{collections::HashMap, time::Duration};
use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use self::frame::Frame;

use super::{ordering::OrderedMessages, reliability::{ReliabilityState, ReliablePackets}};
pub(crate) use packing::PackingScratch;
pub(crate) use systems::{
    established_packet_reader_system,
    established_packet_builder_system,
};


#[derive(Component)]
pub(crate) struct Established {
    reliable_timeout: Duration,
    reliability: ReliablePackets,
    frames: Vec<Frame>,
    ordering: HashMap<ChannelId, OrderedMessages>,
    errors: u32,
}

impl Established {
    pub(in super::super) fn new(
        packet_size: usize,
        reliability: &ReliabilityState,
    ) -> Self {
        Self {
            reliable_timeout: Duration::from_millis(1000), // TODO: Make this a dynamic value based off RTT
            reliability: ReliablePackets::new(reliability.clone()),
            frames: Vec::with_capacity(4),
            ordering: HashMap::default(),
            errors: 0,
        }
    }

    pub fn ordering_entry<'a>(&mut self, channel: ChannelId, cdata: impl Fn() -> &'a ChannelData + 'a) -> &mut OrderedMessages {
        self.ordering.entry(channel)
            .or_insert(match cdata().ordered {
                OrderingGuarantee::Unordered => panic!(),
                OrderingGuarantee::Sequenced => OrderedMessages::sequenced(),
                OrderingGuarantee::Ordered => OrderedMessages::ordered(),
            })
    }
}