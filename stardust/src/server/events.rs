use bevy::prelude::*;

#[derive(Event)]
pub struct ClientConnectedEvent {
    pub id: Entity,
}