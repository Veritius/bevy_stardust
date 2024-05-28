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

impl IntermediateState for InitiatorHello {
    type Next = Completed;

    fn recv_packet(&mut self, shared: &mut HandshakeShared, bytes: Bytes) -> bool {
        todo!()
    }

    fn transition(self, shared: &HandshakeShared) -> Result<Self::Next, Terminated> {
        todo!()
    }
}