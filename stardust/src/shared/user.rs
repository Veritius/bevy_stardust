use bevy::reflect::Reflect;

/// Opaque type used to identify a network peer.
#[derive(Debug, Reflect, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NetworkUserId(pub(crate) u32);
impl NetworkUserId {
    const SERVER: Self = NetworkUserId(0);

    /// Returns whether or not this is the server's ID.
    pub const fn is_server(&self) -> bool {
        self.0 == 0
    }
    
    /// Returns whether or not this is a client's ID.
    pub const fn is_client(&self) -> bool {
        self.0 != 0
    }
}