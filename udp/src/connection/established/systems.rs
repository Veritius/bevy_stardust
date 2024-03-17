use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use untrusted::*;
use crate::{plugin::PluginConfiguration, Connection};
use super::{frame::PacketFrameId, Established, QueuedMessage};

macro_rules! try_unwrap {
    ($id:tt, $st:expr) => {
        match $st {
            Ok(val) => val,
            Err(_) => { continue $id; }
        }
    };
}

pub(crate) fn established_packet_reader_system(
    mut connections: Query<(&mut Connection, &mut Established)>,
) {
    // Process all connections in parallel
    connections.par_iter_mut().for_each(|(mut meta, mut state)| {
        // Drain the message queue to process it
        while let Some(packet) = meta.packet_queue.pop_incoming() {
            let mut reader = Reader::new(Input::from(&packet.payload));

            // System for processing individual frames within a packet.
            'frame: loop {
                // Try to read the header to know what we're working with
                let hbyte = try_unwrap!('frame, reader.read_byte());
                let header_id = try_unwrap!('frame, PacketFrameId::try_from(hbyte));
            }

            todo!()
        }
    });
}

pub(crate) fn established_post_read_queuing_system(
    mut incoming: NetworkIncomingWriter,
    mut connections: Query<(&mut Connection, &mut Established)>,
) {
    todo!()
}

pub(crate) fn established_pre_build_queuing_system(
    outgoing: NetworkOutgoingReader,
    registry: Res<ChannelRegistry>,
    mut connections: Query<(&mut Connection, &mut Established)>,
) {
    let mut iter = outgoing.iter_all();
    while let Some((channel, origin, payload)) = iter.next() {
        if let Ok((_meta, mut state)) = connections.get_mut(origin) {
            state.queue.insert(QueuedMessage {
                priority: registry.get_from_id(channel).unwrap().priority,
                channel,
                payload: payload.clone(),
            });
        }
    }
}

pub(crate) fn established_packet_builder_system(
    config: Res<PluginConfiguration>,
    registry: Res<ChannelRegistry>,
    mut connections: Query<(&mut Connection, &mut Established)>,
) {
    let step = registry.channel_count() / config.reliable_channel_count as u32;

    // Process all connections in parallel
    connections.par_iter_mut().for_each(|(mut meta, mut state)| {
        // Drain the message queue
        while let Some(message) = state.queue.pop_last() {
            // Get the river of the channel
            let river_id = Into::<u32>::into(message.channel) / step;
            let mut river_ref = match river_id {
                0 => &mut state.master,
                _ => &mut state.rivers[river_id as usize],
            };
        }

        todo!()
    });
}