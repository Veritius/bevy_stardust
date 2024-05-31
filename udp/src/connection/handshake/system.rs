use bevy::prelude::*;
use bevy_stardust::prelude::*;
use unbytes::Reader;
use crate::{plugin::PluginConfiguration, sequences::SequenceId};
use self::messages::*;
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

            match (handshake.state.clone(), handshake.direction) {
                (HandshakeState::Hello, Direction::Initiator) => {
                    let message = match ListenerHello::recv(&mut reader) {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    match message {
                        ListenerHello::Rejected(rejection) => {
                            handshake.terminate(
                                rejection.code,
                                Some(rejection.message),
                            );
                        },

                        ListenerHello::Accepted {
                            tpt_ver,
                            app_ver,
                            ack_seq,
                            ack_bits,
                        } => todo!(),
                    }
                },

                (HandshakeState::Hello, Direction::Listener) => {
                    let message = match InitiatorFinish::recv(&mut reader) {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    match message {
                        InitiatorFinish::Rejected(rejection) => {
                            handshake.terminate(
                                rejection.code,
                                Some(rejection.message),
                            );
                        },

                        InitiatorFinish::Accepted {
                            ack_seq,
                            ack_bits,
                        } => todo!(),
                    }
                },

                (HandshakeState::Completed, _) => todo!(),
                (HandshakeState::Terminated(_), _) => todo!(),
            }
        }
    });
}   