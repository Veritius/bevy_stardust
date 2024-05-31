use bevy::prelude::*;
use bevy_stardust::prelude::*;
use unbytes::Reader;
use crate::{plugin::PluginConfiguration, sequences::SequenceId};
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

            // this is a hideous workaround to use the ? operator
            // TODO: Replace when try_trait_v2 stabilises
            if (|| {
                // Read the packet sequence identifier
                let seq: SequenceId = reader.read_u16().map_err(|_| ())?.into();

                // If the packet is too old ignore it
                if seq <= handshake.reliability.remote_sequence {
                    return Err(());
                }

                Ok(())
            })().is_err() { continue };

            match (handshake.state, handshake.direction) {
                (HandshakeState::Hello, Direction::Initiator) => todo!(),
                (HandshakeState::Hello, Direction::Listener) => todo!(),

                (HandshakeState::Completed, _) => todo!(),
                (HandshakeState::Terminated, _) => todo!(),
            }
        }
    });
}   