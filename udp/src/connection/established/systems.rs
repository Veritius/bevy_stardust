use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use crate::{packet::MTU_SIZE, plugin::PluginConfiguration, Connection};
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
        // Scratch space
        let mut scratch = BytesMut::with_capacity(MTU_SIZE);

        // Include an alt message for every N main messages
        let msg_queue_len = outgoing.count();
        let alt_queue_len = state.queue.len();
        let msg_alt_nfrac = msg_queue_len / alt_queue_len;

        // Iterator for individual messages and their channel ids
        let mut msg_queue = outgoing
            .all_queues()
            .flat_map(|(c,s)| s.iter().map(move |v| (c,v)));

        // Iterator for queued messages
        let mut alt_queue = state.queue.iter();

        todo!()
    });
}