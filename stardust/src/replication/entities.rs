use bevy::reflect::Reflect;

/// A unique entity replicated over the network.
#[derive(Debug, Reflect, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReplicatedEntityId(pub(crate) u64);