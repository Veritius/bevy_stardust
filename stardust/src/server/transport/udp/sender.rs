use bevy::prelude::*;
use crate::{shared::channels::{registry::ChannelRegistry, id::ChannelId, components::*}, server::clients::Client};
use super::UdpClient;

pub(super) fn send_packets_system(
    world: &mut World,
) {

}