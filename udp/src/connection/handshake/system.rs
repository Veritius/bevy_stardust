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
            let mut reader = Reader::new(packet);

            // // Read the packet sequence identifier
            // let seq: SequenceId = reader.read_u16()?.into();

            // // If the packet is too old ignore it
            // if seq <= this.shared.reliability.remote_sequence {
            //     return Err(ParseError::Outdated);
            // }
        }
    });
}   