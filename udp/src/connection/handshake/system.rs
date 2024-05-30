use std::mem::swap;

use bevy::prelude::*;
use bevy_stardust::prelude::*;
use unbytes::Reader;
use crate::plugin::PluginConfiguration;
use self::{codes::HandshakeResponseCode, parse::parse_header, terminated::{TerminationOrigin, TerminationReason}};

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

            // Try to parse the packet header
            if let Err(error) = parse_header(&mut handshake, &mut reader) {
                handshake.state = HandshakeState::Terminated(Terminated::from(TerminationReason {
                    code: HandshakeResponseCode::MalformedPacket,
                    origin: TerminationOrigin::Local,
                }));
            }

            let mut state = HandshakeState::Swapping;
            swap(&mut handshake.state, &mut state);

            state = match state {
                HandshakeState::InitiatorHello(state) => match state.recv_packet(&mut handshake.shared, &mut reader) {
                    TransitionOutcome::None(state) => HandshakeState::InitiatorHello(state),
                    TransitionOutcome::Next(state) => HandshakeState::Completed(state),
                    TransitionOutcome::Fail(state) => HandshakeState::Terminated(state),
                },

                HandshakeState::ListenerHello(state) => match state.recv_packet(&mut handshake.shared, &mut reader) {
                    TransitionOutcome::None(state) => HandshakeState::ListenerHello(state),
                    TransitionOutcome::Next(state) => HandshakeState::Completed(state),
                    TransitionOutcome::Fail(state) => HandshakeState::Terminated(state),
                },

                HandshakeState::Completed(_) => state,
                HandshakeState::Terminated(_) => state,
                HandshakeState::Swapping => panic!(),
            };

            swap(&mut handshake.state, &mut state);
        }
    });
}   