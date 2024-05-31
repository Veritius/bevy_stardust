use bevy::prelude::*;
use unbytes::Reader;
use crate::{plugin::PluginConfiguration, sequences::SequenceId, version::{BANNED_MINOR_VERSIONS, TRANSPORT_VERSION_DATA}};
use self::messages::*;
use super::*;

pub(in crate::connection) fn handshake_polling_system(
    config: Res<PluginConfiguration>,
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
                            ); break;
                        },

                        ListenerHello::Accepted {
                            tpt_ver,
                            app_ver,
                            ack_seq,
                            ack_bits,
                        } => {
                            if let Err(code) = version_pair_check(tpt_ver, app_ver, &config) {
                                handshake.terminate(code, None);
                                break;
                            }

                            let _ = handshake.reliability.ack_bits(ack_seq, ack_bits, 2);
                            handshake.change_state(HandshakeState::Completed);
                            break;
                        }
                    }
                },

                (HandshakeState::Hello, Direction::Listener) => {
                    let message = match InitiatorFinish::recv(&mut reader) {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    match message {
                        InitiatorFinish::Rejected(rejection) => {
                            handshake.terminate(rejection.code, Some(rejection.message));
                            break;
                        },

                        InitiatorFinish::Accepted {
                            ack_seq,
                            ack_bits,
                        } => {
                            let _ = handshake.reliability.ack_bits(ack_seq, ack_bits, 2);
                            handshake.change_state(HandshakeState::Completed);
                            break;
                        },
                    }
                },

                // Do nothing.
                (HandshakeState::Completed, _) => {},
                (HandshakeState::Terminated(_), _) => {},
            }
        }
    });
} 

pub(in crate::connection) fn handshake_sending_system(
    config: Res<PluginConfiguration>,
    mut connections: Query<(Entity, &mut Connection, &mut Handshaking)>,
    commands: ParallelCommands,
) {
    // Iterate connections in parallel
    connections.par_iter_mut().for_each(|(entity, mut connection, mut handshake)| {
        // Calculate whether a message needs to be sent
        let send_due = {
            let resend = match handshake.last_sent {
                Some(v) => { v.elapsed() >= RESEND_TIMEOUT } ,
                None => true,
            };

            match (handshake.scflag, resend) {
                (true, _) => true,
                (_, v) => v,
            }
        };

        // If nothing is due to send, return
        if !send_due { return }

        // Scratch space for our messaging
        let mut buf: Vec<u8> = Vec::with_capacity(32);

        match (handshake.state.clone(), handshake.direction) {
            (HandshakeState::Hello, Direction::Initiator) => {
                todo!()
            },

            (HandshakeState::Hello, Direction::Listener) => {
                todo!()
            },

            (HandshakeState::Completed, Direction::Initiator) => {
                todo!()
            },

            (HandshakeState::Completed, Direction::Listener) => {
                todo!()
            },

            (HandshakeState::Terminated(_), _) => todo!(),
        }
    });
}

fn version_pair_check(
    tpt_ver: AppVersion,
    app_ver: AppVersion,
    config: &PluginConfiguration,
) -> Result<(), HandshakeResponseCode> {
    use HandshakeResponseCode::*;
    use crate::version::IncompatibilityReason::*;

    let tpt_chk = TRANSPORT_VERSION_DATA.compare(&tpt_ver, BANNED_MINOR_VERSIONS);
    if let Err(reason) = tpt_chk {
        return Err(match reason {
            MismatchedIdentifier => IncompatibleTransportIdentifier,
            MismatchedMajorVersion => IncompatibleApplicationMajorVersion,
            DeniedMinorVersion => IncompatibleTransportMinorVersion,
        });
    }

    let app_chk = config.application_version.compare(&app_ver, config.denied_minor_versions);
    if let Err(reason) = app_chk {
        return Err(match reason {
            MismatchedIdentifier => IncompatibleApplicationIdentifier,
            MismatchedMajorVersion => IncompatibleApplicationMajorVersion,
            DeniedMinorVersion => IncompatibleApplicationMinorVersion,
        });
    }

    return Ok(());
}