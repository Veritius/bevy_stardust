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

impl Transition for ListenerHello {
    type Next = Completed;

    fn recv_packet(&mut self, shared: &mut HandshakeShared, bytes: Bytes) -> bool {
        todo!()
    }

    fn poll_send(&mut self, shared: &mut HandshakeShared) -> Option<Bytes> {
        todo!()
    }

    fn transition(self, shared: &HandshakeShared) -> Result<Self::Next, Terminated> {
        todo!()
    }
}