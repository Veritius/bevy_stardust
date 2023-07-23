use bevy::reflect::Reflect;

/// Unique identifier for a client.
#[derive(Debug, Reflect, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NetworkUserId(pub(crate) u32);

/// A type that is replicated over the network.
#[derive(Debug, Reflect, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NetworkTypeId(pub(crate) u32);

/// A unique entity replicated over the network.
#[derive(Debug, Reflect, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NetworkEntityId(pub(crate) u64);