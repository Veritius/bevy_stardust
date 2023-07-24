use bevy::prelude::*;
use crate::shared::scheduling::NetworkPostUpdate;
use super::clients::client_comp_despawn_disconnection_system;

pub struct StardustServerPlugin {
    pub port: u16,
}

impl Plugin for StardustServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(NetworkPostUpdate, client_comp_despawn_disconnection_system);
    }
}