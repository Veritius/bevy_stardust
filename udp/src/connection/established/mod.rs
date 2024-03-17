mod river;
mod packing;
mod frame;
mod systems;

pub(crate) use systems::{
    established_breaking_system,
    established_packing_system,
};

use bevy_ecs::prelude::*;
use self::river::River;
use super::reliability::ReliabilityState;

#[derive(Component)]
pub(crate) struct Established {
    master: River,
    rivers: Vec<River>,
}

impl Established {
    pub(in super::super) fn new(
        river_count: u8,
        pk_size: usize,
        rel_state: &ReliabilityState
    ) -> Self {
        // Create river state storage thingies
        let rivers = (0..=river_count)
            .into_iter()
            // Add 1 to id because id 0 is reserved by the master river
            .map(|seq| River::new(seq.saturating_add(1), pk_size, rel_state.clone()))
            .collect::<Vec<_>>();

        Self {
            master: River::new(0, pk_size, rel_state.clone()),
            rivers,
        }
    }
}