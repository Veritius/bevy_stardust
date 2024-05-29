use bytes::Bytes;
use super::*;

pub(super) struct ListenerHello {
    p: ()
}

impl ListenerHello {
    pub fn new() -> Self {
        Self {
            p: ()
        }
    }
}

impl IntermediateState for ListenerHello {
    type Next = Completed;

    fn recv_packet(&mut self, shared: &mut HandshakeShared, bytes: Bytes) -> bool {
        todo!()
    }

    fn transition(self, shared: &HandshakeShared) -> Result<Self::Next, Terminated> {
        todo!()
    }
}