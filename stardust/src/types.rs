use bevy::reflect::Reflect;

/// Opaque type used to identify a network peer.
#[derive(Debug, Reflect, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NetworkUserId(pub(crate) u32);
impl NetworkUserId {
    /// Returns whether or not this is the server's ID.
    pub const fn is_server(&self) -> bool {
        self.0 == 0
    }
    
    /// Returns whether or not this is a client's ID.
    pub const fn is_client(&self) -> bool {
        self.0 != 0
    }
}

/// A type that is replicated over the network.
#[derive(Debug, Reflect, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NetworkTypeId(pub(crate) u32);

/// A unique entity replicated over the network.
#[derive(Debug, Reflect, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NetworkEntityId(pub(crate) u64);