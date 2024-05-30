use std::mem::swap;

use unbytes::*;
use crate::{connection::handshake::HandshakeShared, sequences::SequenceId};
use super::{HandshakeState, Handshaking, Transition};

impl Handshaking {
    pub(super) fn recv_packet(&mut self, reader: Reader) {
        let reader = match parse_header(self, reader) {
            Ok(v) => v,
            Err(err) => {
                todo!()
            },
        };

        let mut state = HandshakeState::Swapping;
        swap(&mut self.state, &mut state);

        #[inline]
        fn state_recv_packet<T: Transition>(
            reader: Reader,
            shared: &mut HandshakeShared,
            mut state: T,
            same: impl Fn(T) -> HandshakeState,
            next: impl Fn(T::Next) -> HandshakeState,
        ) -> HandshakeState {
            state.recv_packet(shared, reader);
            if state.wants_transition(shared) {
                match state.perform_transition(&shared) {
                    Ok(state) => next(state),
                    Err(state) => HandshakeState::Terminated(state),
                }
            } else {
                same(state)
            }
        }

        state = match state {
            HandshakeState::InitiatorHello(state) => {
                state_recv_packet(reader, &mut self.shared, state,
                    |state| HandshakeState::InitiatorHello(state),
                    |state| HandshakeState::Completed(state),
                )
            },

            HandshakeState::ListenerHello(state) => {
                state_recv_packet(reader, &mut self.shared, state,
                    |state| HandshakeState::ListenerHello(state),
                    |state| HandshakeState::Completed(state),
                )
            }

            HandshakeState::Completed(_) => state,
            HandshakeState::Terminated(_) => state,

            HandshakeState::Swapping => unreachable!(),
        };

        swap(&mut self.state, &mut state);
        drop(state);
    }
}

fn parse_header(
    this: &mut Handshaking,
    mut reader: Reader,
) -> Result<Reader, ParseError> {
    // Read the packet sequence identifier
    let seq: SequenceId = reader.read_u16()?.into();

    // If the packet is too old ignore it
    if seq <= this.shared.reliability.remote_sequence {
        return Err(ParseError::Outdated);
    }

    return Ok(reader);
}

enum ParseError {
    EndOfInput(EndOfInput),
    Outdated,
}

impl From<EndOfInput> for ParseError {
    fn from(value: EndOfInput) -> Self {
        Self::EndOfInput(value)
    }
}