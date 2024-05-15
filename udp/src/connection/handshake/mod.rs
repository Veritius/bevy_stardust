mod codes;
mod packets;

pub(in crate::connection) use codes::HandshakeResponseCode;

use bytes::Bytes;
use crate::plugin::PluginConfiguration;
use super::{shared::ConnectionShared, ConnectionDirection};

pub(super) struct HandshakeStateMachine {
    state: HandshakeStateInner,
}

impl HandshakeStateMachine {
    pub fn new(direction: ConnectionDirection) -> Self {
        Self {
            state: match direction {
                ConnectionDirection::Client => HandshakeStateInner::InitiatorHello,
                ConnectionDirection::Server => HandshakeStateInner::ListenerResponse,
            },
        }
    }

    pub fn recv(
        &mut self,
        packet: Bytes,
        config: &PluginConfiguration,
        shared: &mut ConnectionShared,
    ) -> Option<HandshakeOutcome> {
        match self.state {
            HandshakeStateInner::InitiatorHello => todo!(),
            HandshakeStateInner::ListenerResponse => todo!(),
            HandshakeStateInner::InitiatorResponse => todo!(),
        }
    }
}

#[derive(Clone, Copy)]
enum HandshakeStateInner {
    InitiatorHello,
    ListenerResponse,
    InitiatorResponse,
}

pub(super) enum HandshakeOutcome {
    FinishedHandshake,
    FailedHandshake,
}