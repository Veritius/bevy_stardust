mod codes;
mod packets;

pub(in crate::connection) use codes::HandshakeResponseCode;

use bytes::Bytes;
use std::collections::BTreeMap;
use crate::{plugin::PluginConfiguration, sequences::SequenceId};
use super::{reliability::*, ConnectionDirection};

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
        conifg: &PluginConfiguration,
        reliability: &mut ReliabilityState,
        packets: &mut BTreeMap<SequenceId, UnackedPacket>,
    ) -> HandshakeRecvOutcome {
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

pub(super) enum HandshakeRecvOutcome {
    FinishedHandshake,
    FailedHandshake,
}