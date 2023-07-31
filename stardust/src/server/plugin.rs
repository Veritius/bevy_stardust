use bevy::prelude::*;
use super::connection::{TryDisconnectEvent, PlayerConnectedEvent, PlayerDisconnectedEvent};

pub struct StardustServerPlugin;

impl Plugin for StardustServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TryDisconnectEvent>();
        app.add_event::<PlayerConnectedEvent>();
        app.add_event::<PlayerDisconnectedEvent>();
    }
}