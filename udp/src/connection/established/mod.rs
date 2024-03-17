mod river;
mod packing;
mod frame;
mod systems;

use bevy_stardust::channels::id::ChannelId;
use bytes::Bytes;
pub(crate) use systems::{
    established_packet_reader_system,
    established_post_read_queuing_system,
    established_pre_build_queuing_system,
    established_packet_builder_system,
};

use std::collections::BTreeSet;
use bevy_ecs::prelude::*;
use self::river::River;
use super::reliability::ReliabilityState;

#[derive(Component)]
pub(crate) struct Established {
    queue: BTreeSet<QueuedMessage>,
    master: River,
    rivers: Vec<River>,
}

impl Established {
    pub(in super::super) fn new(
        river_count: u8,
        pk_size: usize,
        rel_state: &ReliabilityState,
    ) -> Self {
        // Create river state storage thingies
        let rivers = (0..=river_count)
            .into_iter()
            // Add 1 to id because id 0 is reserved by the master river
            .map(|seq| River::new(seq.saturating_add(1), pk_size, rel_state.clone()))
            .collect::<Vec<_>>();

        Self {
            queue: BTreeSet::new(),
            master: River::new(0, pk_size, rel_state.clone()),
            rivers,
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