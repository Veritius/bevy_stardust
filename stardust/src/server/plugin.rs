use bevy::prelude::*;
use crate::shared::scheduling::ReadPackets;

use super::receive::receive_packets_system;

pub struct StardustServerPlugin {
    pub port: u16,
}

impl Plugin for StardustServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(ReadPackets, receive_packets_system);
    }
}