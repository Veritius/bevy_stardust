//! Connection management systemparam and events.

use bevy::{prelude::*, ecs::system::SystemParam};

#[derive(SystemParam)]
pub struct ConnectionManager<'w> {
    disconnect_events: EventWriter<'w, TryDisconnectEvent>,
}

impl ConnectionManager<'_> {
    /// Disconnects a user from the server.
    pub fn disconnect_user(&mut self, target: Entity) {
        self.disconnect_events.send(TryDisconnectEvent(target));
    }
}

/// Raise to disconnect a client. This should be processed by the transport layer.
#[derive(Event)]
pub struct TryDisconnectEvent(pub Entity);

/// Raised when a new player connects.
#[derive(Event)]
pub struct PlayerConnectedEvent(pub Entity);

/// Raised when a player disconnects.
#[derive(Event)]
pub struct PlayerDisconnectedEvent;