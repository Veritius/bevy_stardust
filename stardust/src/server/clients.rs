//! Components for client entities.

use std::time::Instant;
use bevy::prelude::*;

/// Represents a connected client as an entity.
/// 
/// Despawning the entity or otherwise removing the component will silently disconnect the client.
#[derive(Debug, Component)]
pub struct Client {
    /// The time this client joined.
    pub joined: Instant,
}

impl Client {
    pub fn new() -> Self {
        Self {
            joined: Instant::now(),
        }
    }
}