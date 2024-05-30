use bevy::prelude::*;
use bevy_stardust::prelude::*;
use unbytes::Reader;
use crate::plugin::PluginConfiguration;
use super::*;

pub(crate) fn handshake_polling_system(
    registry: Res<ChannelRegistry>,
    config: Res<PluginConfiguration>,
    commands: ParallelCommands,
    mut connections: Query<(Entity, &mut Connection, &mut Handshaking)>,
) {
    // Iterate connections in parallel
    connections.par_iter_mut().for_each(|(entity, mut connection, mut handshake)| {
        // Read packets from the receive queue into the handshaking component
        while let Some(packet) = connection.recv_queue.pop_front() {
            // handshake.recv_packet(Reader::new(packet));
        }
    });
}   