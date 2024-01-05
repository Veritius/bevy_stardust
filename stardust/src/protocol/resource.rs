use std::hash::Hasher;
use bevy::prelude::*;
use gxhash::GxHasher;
use super::stablehash::STABLE_HASHER_SEED;

/// A unique value generated during `App` creation, used to ensure two clients have consistent network setups.
#[derive(Resource)]
pub struct ProtocolConfigHash {
    int: u64,
    hex: [u8; 16],
}

impl ProtocolConfigHash {

}

/// Stores the state of the hasher before a result is finalized
#[derive(Resource)]
pub(crate) struct PendingHashValues {
    pub state: Box<dyn Hasher + Send + Sync + 'static>,
}

impl PendingHashValues {
    pub fn new() -> Self {
        Self {
            state: Box::new(GxHasher::with_seed(STABLE_HASHER_SEED)),
        }
    }
}

/// Adds the `UniqueNetworkHash` resource to the world.
pub fn finalise_hasher_system(
    world: &mut World
) {
    // Remove hasher resource
    let mut hasher = world.remove_resource::<PendingHashValues>().unwrap();

    // Get hasher values
    let int = hasher.state.finish();
    let hex = format!("{:X}", int).as_bytes().try_into().unwrap();

    // Insert hash resource
    world.insert_resource(ProtocolConfigHash { int, hex });
}