//! The "safety net" of Stardust, used to prevent weird and hard-to-find issues.
//! By creating a hashed value at startup from networking-related actions, like adding channels, hard to debug issues can be effectively prevented.

use bevy::prelude::*;
#[allow(deprecated)] // Intentional - keeps hasher implementation the same through Rust releases.
use std::hash::{Hash, Hasher, SipHasher};

/// A unique value generated during `App` creation, used to ensure two clients have consistent network setups.
#[derive(Resource)]
pub struct UniqueNetworkHash {
    pub(super) int: u64,
    pub(super) hex: String,
}

impl UniqueNetworkHash {
    /// Returns the integer form of the network hash.
    pub fn int(&self) -> u64 {
        self.int
    }

    /// Returns the hexadecimal form of the network hash as a string.
    pub fn hex(&self) -> &str {
        &self.hex
    }
}

/// Extends Bevy's `App` struct to make creating a [UniqueNetworkHash].
/// Don't implement this yourself.
pub trait NetworkHashAppExt {
    /// Add a value to adjust the [UniqueNetworkHash].
    /// 
    /// Using this function depends on the ordering of its use. `f(A) f(B)` has a different result to `f(B) f(A)`.
    /// If you don't want this, use `net_hash_string`.
    fn net_hash_value(&mut self, value: impl Hash);

    /// Add a string to adjust the [UniqueNetworkHash].
    /// 
    /// You can use this if you insert values in an unpredictable order, but want a consistent output.
    /// If you want to put in any value, or ordering matters, use `net_hash_value`.
    fn net_hash_string(&mut self, value: impl Into<String>);
}

impl NetworkHashAppExt for App {
    fn net_hash_value(&mut self, value: impl Hash) {
        let mut hasher = self.world.resource_mut::<UniqueNetworkHasher>();
        value.hash(&mut hasher.state);
    }

    fn net_hash_string(&mut self, value: impl Into<String>) {
        let mut hasher = self.world.resource_mut::<UniqueNetworkHasher>();
        hasher.strings.push(value.into());
    }
}

/// Stores the state of the hasher before a result is finalized
#[derive(Resource)]
pub(super) struct UniqueNetworkHasher {
    strings: Vec<String>,
    state: Box<dyn Hasher + Send + Sync + 'static>,
}

impl UniqueNetworkHasher {
    pub fn new() -> Self {
        Self {
            strings: vec![],
            #[allow(deprecated)]
            state: Box::new(Box::new(SipHasher::default())),
        }
    }
}

/// Adds the `UniqueNetworkHash` resource to the world.
pub fn complete_hasher(
    world: &mut World
) {
    // Remove hasher resource
    let mut hasher = world.remove_resource::<UniqueNetworkHasher>().unwrap();
    
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
    world.insert_resource(UniqueNetworkHash { int, hex });
}