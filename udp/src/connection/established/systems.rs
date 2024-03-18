use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use crate::{plugin::PluginConfiguration, Connection};
use super::Established;

macro_rules! try_unwrap {
    ($id:tt, $st:expr) => {
        match $st {
            Ok(val) => val,
            Err(_) => { continue $id; }
        }
    };
}

pub(crate) fn established_packet_reader_system(
    mut connections: Query<(&mut Connection, &mut Established, &mut NetworkMessages<Incoming>)>,
) {
    // Process all connections in parallel
    connections.par_iter_mut().for_each(|(mut meta, mut state, mut incoming)| {
        todo!()
    });
}

pub(crate) fn established_packet_builder_system(
    registry: ChannelRegistry,
    config: Res<PluginConfiguration>,
    mut connections: Query<(&mut Connection, &mut Established, &NetworkMessages<Outgoing>)>,
) {
    // Process all connections in parallel
    connections.par_iter_mut().for_each(|(mut meta, mut state, outgoing)| {
        // Categorisations of various messages
        // TODO: Instead of reallocating every time, create some kind of persistent buffer allocation that is passed between threads.
        let mut reliable_messages = Vec::new();
        let mut unreliable_messages = Vec::new();

        // Iterate over all queues to collect messages
        let mut queues = outgoing.all_queues();
        while let Some((channel, payloads)) = queues.next() {
            // Get channel data
            let channel_data = registry.channel_config(channel).unwrap();
            let is_reliable = channel_data.reliable == ReliabilityGuarantee::Reliable;

            // Iterate over all payloads
            for payload in payloads {
                match is_reliable {
                    true => reliable_messages.push((channel, payload)),
                    false => unreliable_messages.push((channel, payload)),
                }
            }
        }

        todo!()
    });
}