use bytes::Bytes;
use super::*;

pub(super) struct ListenerHello {
    _hidden: ()
}

impl ListenerHello {
    pub fn new() -> Self {
        Self {
            _hidden: ()
        }
    }
}

impl Transition for ListenerHello {
    type Next = Completed;

    fn recv_packet(&mut self, shared: &mut HandshakeShared, bytes: Reader) {
        todo!()
    }

    fn poll_send(&mut self, shared: &mut HandshakeShared) -> Option<Bytes> {
        todo!()
    }

    fn wants_transition(&self, shared: &HandshakeShared) -> bool {
        todo!()
    }

    fn perform_transition(self, shared: &HandshakeShared) -> Result<Self::Next, Terminated> {
        todo!()
    }
}