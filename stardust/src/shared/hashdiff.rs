use bevy::prelude::*;

/// A unique value generated when adding things like channels.
/// This is used to compare two peers when connecting, to prevent different versions of the game from playing together.
#[derive(Resource)]
pub struct UniqueNetworkHash(pub(super) u64);

impl UniqueNetworkHash {
    pub fn inner(&self) -> u64 {
        self.0
    }
}

pub trait NetworkHashAppExt {
    /// Adds a hashable value to change the [UniqueNetworkHash].
    /// The reason this exists is to prevent clients/servers from connecting when they have different versions of the game.
    fn add_net_hash_value(&mut self, value: impl std::hash::Hash);
}

impl NetworkHashAppExt for App {
    fn add_net_hash_value(&mut self, value: impl std::hash::Hash) {
        let mut hasher = self.world.resource_mut::<UniqueNetworkHasher>();
        value.hash(&mut hasher.0);
    }
}

/// Stores the state of the hasher before a result is finalized
#[derive(Resource)]
pub(super) struct UniqueNetworkHasher(pub Box<dyn std::hash::Hasher + Send + Sync + 'static>);