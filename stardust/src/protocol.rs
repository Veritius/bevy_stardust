//! The "safety net" of Stardust, used to prevent weird and hard-to-find issues.
//! By creating a hashed value at startup from networking-related actions, like adding channels, hard to debug issues can be effectively prevented.

// TODO: Don't use AHash, it's not stable across compilations, Rust versions, or platforms
use bevy::{prelude::*, utils::AHasher};
use std::hash::{Hash, Hasher};

/// A unique value generated during `App` creation, used to ensure two clients have consistent network setups.
#[derive(Resource)]
pub struct ProtocolId {
    int: u64,
    hex: String,
}

impl ProtocolId {
    /// Returns the integer form of the network hash.
    pub fn int(&self) -> u64 {
        self.int
    }

    /// Returns the hexadecimal form of the network hash as a string.
    pub fn hex(&self) -> &str {
        &self.hex
    }
}

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
    fn net_hash_value(&mut self, value: impl Hash);

    /// Stores `value` for later sorting and hashing.
    /// 
    /// You can use this if you insert values in an unpredictable order, but want a consistent output.
    /// If you want to put in any value, or ordering matters, use `net_hash_value`.
    fn net_hash_string(&mut self, value: impl Into<String>);
}

impl ProtocolIdAppExt for App {
    fn net_hash_value(&mut self, value: impl Hash) {
        let mut hasher = self.world.resource_mut::<ProtocolIdHasher>();
        value.hash(&mut hasher.state);
    }

    fn net_hash_string(&mut self, value: impl Into<String>) {
        let mut hasher = self.world.resource_mut::<ProtocolIdHasher>();
        hasher.strings.push(value.into());
    }
}

/// Stores the state of the hasher before a result is finalized
#[derive(Resource)]
pub(super) struct ProtocolIdHasher {
    strings: Vec<String>,
    state: Box<dyn Hasher + Send + Sync + 'static>,
}

impl ProtocolIdHasher {
    pub fn new() -> Self {
        Self {
            strings: vec![],
            state: Box::new(AHasher::default()),
        }
    }
}

/// Adds the `UniqueNetworkHash` resource to the world.
pub fn complete_hasher(
    world: &mut World
) {
    // Remove hasher resource
    let mut hasher = world.remove_resource::<ProtocolIdHasher>().unwrap();
    
    // Sort and add strings
    hasher.strings.sort();
    loop {
        let Some(string) = hasher.strings.pop() else { break; };
        string.hash(&mut hasher.state);
    }

    // Get hasher values
    let int = hasher.state.finish();
    let hex = format!("{:X}", int);

    // Insert hash resource
    world.insert_resource(ProtocolId { int, hex });
}