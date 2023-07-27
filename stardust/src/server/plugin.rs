use bevy::prelude::*;
use crate::shared::scheduling::{ReadPackets, NetworkPreUpdateCleanup};
use super::{systems::receive_packets_system, receive::clear_channel_data_system};

pub struct StardustServerPlugin {
    pub port: u16,
}

impl Plugin for StardustServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(ReadPackets, receive_packets_system);
        app.add_systems(NetworkPreUpdateCleanup, clear_channel_data_system);
    }
}