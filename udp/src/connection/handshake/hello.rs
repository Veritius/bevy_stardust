use bytes::Bytes;
use hello::terminated::TerminationOrigin;
use self::codes::HandshakeResponseCode;
use super::*;

// TODO: Replace this with the ? operator when it stabilises
macro_rules! try_read {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(_) => { return TransitionOutcome::terminated(TerminationReason {
                code: HandshakeResponseCode::MalformedPacket,
                origin: TerminationOrigin::Local,
            })},
        }
    };
}

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

    fn recv_packet(mut self, shared: &mut HandshakeShared, reader: &mut Reader) -> TransitionOutcome<Self> {
        let transport_version = try_read!(AppVersion::from_bytes(reader));
        let application_version = try_read!(AppVersion::from_bytes(reader));

        todo!()
    }

    fn poll_send(&mut self, shared: &mut HandshakeShared) -> Option<Bytes> {
        todo!()
    }
}