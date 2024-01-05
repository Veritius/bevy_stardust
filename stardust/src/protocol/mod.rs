//! The "safety net" of Stardust, used to prevent weird and hard-to-find issues.
//! By creating a hashed value at startup from networking-related actions, like adding channels, hard to debug issues can be effectively prevented.

mod stablehash;
mod resource;

use bevy::prelude::*;

pub(crate) use resource::{PendingHashValues, finalise_hasher_system};

pub use stablehash::StableHash;
pub use resource::ProtocolConfigHash;

mod sealed {
    pub trait Sealed {}
    impl Sealed for bevy::prelude::App {}
}

/// Extends Bevy's `App` to add methods for generating the [ProtocolId].
pub trait ProtocolIdAppExt: sealed::Sealed {
    /// Hashes `value` immediately.
    /// 
    /// Using this function depends on the ordering of its use. `f(A) f(B)` has a different result to `f(B) f(A)`.
    /// If you don't want this, use `net_hash_string`.
    fn net_hash_value(&mut self, value: impl StableHash);
}

impl ProtocolIdAppExt for App {
    fn net_hash_value(&mut self, value: impl StableHash) {
        let mut hasher = self.world.resource_mut::<PendingHashValues>();
        value.hash(&mut hasher.state);
    }
}