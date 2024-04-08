use bevy::prelude::*;
use bevy_stardust::prelude::*;

/// Defines a 'network room'. This filters the entities that are replicated to each peer.
/// Peers considered members of the room (as per [`NetworkGroup`]) will have entities replicated to them.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct NetworkRoom {
    /// See [`RoomFilterMode`]'s documentation.
    pub mode: RoomFilterMode,
}

/// A bundle for a minimal network room.
#[derive(Bundle)]
#[allow(missing_docs)]
pub struct NetworkRoomBundle {
    pub room: NetworkRoom,
    pub group: NetworkGroup,
}

/// Defines how peers in the room should be filtered out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum RoomFilterMode {
    /// Only peers in the room will have entities replicated to them.
    Exclusive,

    /// Only peers outside of the room will have entities replicated to them.
    Inclusive,
}