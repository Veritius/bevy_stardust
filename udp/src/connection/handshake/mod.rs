mod codes;
mod state;

use self::state::HandshakeStateInner;

/// State of an ongoing handshake.
pub(super) struct HandshakeState {
    state: HandshakeStateInner,
}