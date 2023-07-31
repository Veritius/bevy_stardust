use bevy::prelude::*;
use crate::{shared::channels::registry::ChannelRegistry, server::clients::Client};
use super::UdpClient;

pub(super) fn send_packets_system(
    world: &mut World,
) {
    let channel_registry = world.resource::<ChannelRegistry>();
    let channel_count = channel_registry.channel_count();

    let clients = world.query::<(&Client, &UdpClient)>();
    let components = world.components();
    
    // let res_id = components.get_resource_id(type_id);
    
}