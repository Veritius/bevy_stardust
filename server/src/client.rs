use bevy::{prelude::Component, reflect::Reflect};
use bevy_stardust_shared::types::NetworkUserId;

/// Connected clients, as an entity.
/// Despawning these will disconnect the client.
#[derive(Component, Reflect)]
pub struct Client {
    pub id: NetworkUserId,
}