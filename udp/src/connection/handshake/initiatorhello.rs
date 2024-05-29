use bytes::Bytes;
use super::*;

pub(super) struct InitiatorHello {
    p: ()
}

impl InitiatorHello {
    pub fn new() -> Self {
        Self {
            p: ()
        }
    }
}

impl Transition for InitiatorHello {
    type Next = Completed;

    fn recv_packet(&mut self, shared: &mut HandshakeShared, bytes: Reader) -> bool {
        todo!()
    }

    fn poll_send(&mut self, shared: &mut HandshakeShared) -> Option<Bytes> {
        todo!()
    }

    fn wants_transition(&self, shared: &HandshakeShared) -> bool {
        todo!()
    }

    fn transition(self, shared: &HandshakeShared) -> Result<Self::Next, Terminated> {
        todo!()
    }
}