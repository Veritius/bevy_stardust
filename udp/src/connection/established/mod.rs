mod frame;
mod systems;

use std::collections::HashMap;
use bevy_stardust::prelude::*;
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
    errors: u32,
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

    fn flag_error(&mut self, severity: ErrorSeverity) {
        self.errors += match severity {
            ErrorSeverity::Minor => 1,
            ErrorSeverity::Major => 3,
            ErrorSeverity::Critical => 9,
        }
    }
}

enum ErrorSeverity {
    Minor,
    Major,
    Critical,
}