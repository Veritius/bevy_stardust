use bevy::reflect::Reflect;

/// Unique identifier for a client.
#[derive(Debug, Reflect)]
pub struct NetworkUserId(pub(crate) u32);

/// A type that is replicated over the network.
#[derive(Debug, Reflect)]
pub struct NetworkTypeId(pub(crate) u32);

/// A unique entity replicated over the network.
#[derive(Debug, Reflect)]
pub struct NetworkEntityId(pub(crate) u64);