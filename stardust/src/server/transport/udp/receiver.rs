use std::{collections::BTreeMap, sync::Mutex};
use bevy::{prelude::*, tasks::TaskPool};
use crate::{server::clients::Client, shared::{channels::{components::*, incoming::IncomingNetworkMessages, registry::ChannelRegistry, id::ChannelId}, payload::{Payloads, Payload}}};
use super::{PACKET_HEADER_SIZE, MAX_PACKET_LENGTH, UdpClient, ports::PortBindings};

pub(super) fn receive_packets_system(
    mut clients: Query<(Entity, &Client, &UdpClient, &mut IncomingNetworkMessages)>,
    ports: Res<PortBindings>,
    channels: Query<(&ChannelData, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    channel_registry: Res<ChannelRegistry>,
) {
    // // Create task pool
    // let pool = TaskPool::new();

    // // SAFETY: Panics if the a IncomingNetworkMessages is accessed more than once _throughout the entire system_.
    // struct IncomingNetworkMessageMutabilityManager<'w, 's> {
    //     query: &'w mut Query<'w, 's, (Entity, &'w Client, &'w UdpClient, &'w mut IncomingNetworkMessages)>,
    //     claimed: Vec<Entity>,
    // }

    // let mut manager = Mutex::new(IncomingNetworkMessageMutabilityManager {
    //     query: &mut clients,
    //     claimed: vec![],
    // });

    // pool.scope(|s| {
    //     for port in ports.iter() {
    //         s.spawn(async move {

    //         });
    //     }
    // });
}