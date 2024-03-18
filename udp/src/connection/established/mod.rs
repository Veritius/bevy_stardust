mod packing;
mod frame;
mod systems;

use std::collections::VecDeque;
use bevy_stardust::channels::ChannelId;
use bytes::Bytes;
use bevy_ecs::prelude::*;
use self::packing::PackingManager;
use super::reliability::{ReliabilityState, ReliablePackets};
pub(crate) use systems::{
    established_packet_reader_system,
    established_packet_builder_system,
};


#[derive(Component)]
pub(crate) struct Established {
    queue: VecDeque<QueuedMessage>,
    packer: PackingManager,
    reliability: ReliablePackets,
}

impl Established {
    pub(in super::super) fn new(
        packet_size: usize,
        reliability: &ReliabilityState,
    ) -> Self {
        Self {
            queue: VecDeque::with_capacity(8),
            packer: PackingManager::new(packet_size),
            reliability: ReliablePackets::new(reliability.clone())
        }
    }
}

struct QueuedMessage {
    priority: u32,
    channel: ChannelId,
    payload: Bytes,
}

impl PartialEq for QueuedMessage {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for QueuedMessage {}

impl PartialOrd for QueuedMessage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl Ord for QueuedMessage {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}