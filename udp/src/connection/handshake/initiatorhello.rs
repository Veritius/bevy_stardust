use bytes::Bytes;
use super::*;

pub(super) struct InitiatorHello {
    _hidden: ()
}

impl InitiatorHello {
    pub fn new() -> Self {
        Self {
            _hidden: ()
        }
    }
}

impl Transition for InitiatorHello {
    type Next = Completed;

    fn recv_packet(mut self, shared: &mut HandshakeShared, reader: &mut Reader) -> TransitionOutcome<Self> {
        todo!()
    }

    fn poll_send(&mut self, shared: &mut HandshakeShared) -> Option<Bytes> {
        todo!()
    }
}