use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use crate::messages::outgoing::TransportOutgoingReader;
use crate::prelude::*;
use super::ports::PortBindings;

/// Sends octet strings using a taskpool strategy.
pub(super) fn send_packets_system(
    registry: Res<ChannelRegistry>,
    ports: Res<PortBindings>,
    outgoing: TransportOutgoingReader,
) {
    // Create task pool
    let taskpool = TaskPoolBuilder::new()
        .thread_name("UDP pkt send".to_string())
        .build();
    
    // Iterate over all messages
    taskpool.scope(|s| {
        for (_, socket, peers) in ports.iter() {
            s.spawn(async move {

            });
        }
    });
}