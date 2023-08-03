use bevy::{prelude::*, tasks::TaskPool};
use crate::{shared::channels::{registry::ChannelRegistry, outgoing::OutgoingOctetStringsAccessor}, server::clients::Client};
use super::UdpClient;

pub(super) fn send_packets_system(
    registry: Res<ChannelRegistry>,
    outgoing: OutgoingOctetStringsAccessor,
    clients: Query<(&Client, &UdpClient)>,
) {
    // Create task pool
    let pool = TaskPool::new();

    // Iterate all channels
    let iterator = outgoing.all();
    pool.scope(|s| {
        for x in iterator {
            s.spawn(async move {
                // Access item fields
                let channel = x.id();
                let outgoing = x.octets();

                // Send all outgoing
                let iterator = outgoing.read();
                for (targets, octets) in iterator {
                    
                }
            });
        }
    });
}