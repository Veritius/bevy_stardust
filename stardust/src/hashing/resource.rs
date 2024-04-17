use std::hash::Hasher;
use bevy::prelude::*;
use gxhash::GxHasher;
use super::stablehash::STABLE_HASHER_SEED;

/// A unique value generated during `App` creation, used to ensure two clients have consistent network setups.
/// 
/// Mutating this value through the `Reflect` implementation must be avoided.
/// It's possible, but since this is a computed value, it's bad to have that happen.
#[derive(Debug, Resource, Reflect)]
#[reflect(Debug, Resource)]
pub struct ProtocolConfigHash {
    int: u64,
}

impl ProtocolConfigHash {
    /// Returns the integer representation of the hash.
    pub fn int(&self) -> u64 {
        self.int
    }
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
    let hasher = world.remove_resource::<PendingHashValues>().unwrap();

    // Get hasher values
    let int = hasher.state.finish();

    // Insert hash resource
    world.insert_resource(ProtocolConfigHash { int });
}