use bevy::{prelude::Entity, utils::EntityHashSet};

/// A token of an endpoint 'owning' a connection.
/// 
/// # Safety
/// `ConnectionOwnershipToken`s **may not** be cloned or copied in any way, ever.
/// They are unique to the `World` they were created in, and should never move to another.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(super) struct ConnectionOwnershipToken {
    id: Entity,
}

impl ConnectionOwnershipToken {
    // SAFETY: Only one `ConnectionId` may exist per world.
    pub(super) unsafe fn new(id: Entity) -> Self {
        Self { id }
    }

    pub(super) fn inner(&self) -> Entity {
        self.id
    }
}

impl PartialEq<ConnectionId> for ConnectionOwnershipToken {
    #[inline]
    fn eq(&self, other: &ConnectionId) -> bool {
        self.id == other.id
    }
}

impl PartialEq<ConnectionOwnershipToken> for ConnectionId {
    #[inline]
    fn eq(&self, other: &ConnectionOwnershipToken) -> bool {
        self.id == other.id
    }
}

/// A unique ID for a connection.
/// 
/// # Safety
/// This ID is only unique to the `World` it was created in.
/// You should never allow it to be used in another `World`.
/// To do so will cause undefined behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ConnectionId {
    id: Entity,
}

impl From<&ConnectionOwnershipToken> for ConnectionId {
    fn from(value: &ConnectionOwnershipToken) -> Self {
        Self { id: value.id }
    }
}

pub(crate) struct EndpointConnections {
    owned: EntityHashSet<ConnectionOwnershipToken>,
}

impl EndpointConnections {
    pub(super) fn new() -> Self {
        Self { owned: EntityHashSet::default() }
    }

    pub(super) fn register(&mut self, token: ConnectionOwnershipToken) {
        self.owned.insert(token);
    }

    pub(super) fn deregister(&mut self, id: ConnectionId) {
        // SAFETY: This ConnectionOwnershipToken is dropped after this function
        self.owned.remove(unsafe { &ConnectionOwnershipToken::new(id.id) });
    }

    pub fn contains(&self, id: ConnectionId) -> bool {
        // SAFETY: This ConnectionOwnershipToken is dropped after this function
        self.owned.contains(unsafe { &ConnectionOwnershipToken::new(id.id) })
    }

    pub fn iter(&self) -> impl Iterator<Item = ConnectionId> + '_ {
        self.owned.iter().map(|v| v.into())
    }
}