use bevy::prelude::*;
use crate::shared::scheduling::{TransportReadPackets, NetworkPreUpdateCleanup};
use super::{systems::receive_packets_system, receive::{clear_channel_data_system, AllChannelData}};

pub struct StardustServerPlugin {
    pub port: u16,
}

impl Plugin for StardustServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AllChannelData::default());

        app.add_systems(TransportReadPackets, receive_packets_system);
        app.add_systems(NetworkPreUpdateCleanup, clear_channel_data_system);
    }
}