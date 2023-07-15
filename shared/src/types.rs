/// Unique identifier for a client.
pub struct NetworkUserId(pub(crate) u32);

/// A type that is replicated over the network.
pub struct NetworkTypeId(pub(crate) u32);

/// A unique entity replicated over the network.
pub struct NetworkEntityId(pub(crate) u64);