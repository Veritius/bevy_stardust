use bevy::{prelude::*, tasks::TaskPool};
use crate::{shared::channels::{registry::ChannelRegistry, outgoing::OutgoingOctetStringsAccessor, components::*}, server::clients::Client};
use super::UdpClient;

pub(super) fn send_packets_system(
    registry: Res<ChannelRegistry>,
    channels: Query<(&ChannelData, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    outgoing: OutgoingOctetStringsAccessor,
    clients: Query<(&Client, &UdpClient)>,
) {
    // Create task pool
    let pool = TaskPool::new();

    // Create explicit borrows to prevent moves in taskpool items
    let (registry, channels, clients) = (registry.as_ref(), &channels, &clients);

    // Iterate all channels
    let iterator = outgoing.all();
    pool.scope(|s| {
        for x in iterator {
            s.spawn(async move {
                // Access item fields and channel config
                let channel = x.id();

                let outgoing = x.octets();

                let config = if let Some(entity) = registry.get_from_id(channel) {
                    channels.get(entity).unwrap()
                } else {
                    // Channel doesn't exist, stop this thread
                    error!("Couldn't send message on channel {:?} as it didn't exist", channel);
                    return;
                };

                // Send all outgoing
                let iterator = outgoing.read();
                for (targets, octets) in iterator {
                    
                }
            });
        }
    });
}