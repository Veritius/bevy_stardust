mod packet;
mod systems;

use std::{collections::HashMap, time::Duration};
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use packet::Frame;

use super::{ordering::OrderedMessages, reliability::{ReliabilityState, ReliablePackets}};
pub(crate) use systems::{
    established_packet_reader_system,
    established_packet_builder_system,
    established_timeout_system,
};


#[derive(Component)]
pub(crate) struct Established {
    reliable_timeout: Duration,
    reliability: ReliablePackets,
    frames: Vec<Frame>,
    orderings: HashMap<u32, OrderedMessages>,
    errors: u32,
}

impl Established {
    pub(in super::super) fn new(
        packet_size: usize,
        reliability: &ReliabilityState,
        registry: &ChannelRegistryInner,
    ) -> Self {
        // TODO: Is there a better solution than this?
        // This is done to prevent checks while building packets.
        // Theoretically this is faster, but it's still a little yucky.
        let mut orderings = (0..registry.channel_count())
        .map(|v| {
            let i = v.wrapping_add(1);
            let data = registry.channel_config(ChannelId::from(v)).unwrap();
            let ord = match data.ordered {
                OrderingGuarantee::Unordered => { return None },
                OrderingGuarantee::Sequenced => OrderedMessages::sequenced(),
                OrderingGuarantee::Ordered => OrderedMessages::ordered(),
            };
            Some((i, ord))
        })
        .filter(|v| v.is_some())
        .map(|v| v.unwrap())
        .collect::<HashMap<_, _>>();

        orderings.insert(0, OrderedMessages::ordered());

        Self {
            reliable_timeout: Duration::from_millis(1000), // TODO: Make this a dynamic value based off RTT
            reliability: ReliablePackets::new(reliability.clone()),
            frames: Vec::with_capacity(4),
            orderings,
            errors: 0,
        }
    }

    #[inline]
    pub fn ordering(&mut self, ident: u32) -> &mut OrderedMessages {
        self.orderings.get_mut(&ident).unwrap()
    }
}