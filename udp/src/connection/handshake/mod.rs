mod codes;
mod packets;

pub(in crate::connection) use codes::HandshakeResponseCode;

use bytes::Bytes;
use unbytes::Reader;
use crate::plugin::PluginConfiguration;
use self::packets::HandshakePacketHeader;
use super::{reliability::*, shared::*, ConnectionDirection};

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
        let mut reader = Reader::new(packet);

        let header = match HandshakePacketHeader::read(&mut reader) {
            Ok(header) => header,
            Err(_) => {
                return Some(HandshakeOutcome::FailedHandshake);
            },
        };

        match self.state {
            HandshakeStateInner::InitiatorHello => todo!(),
            HandshakeStateInner::ListenerResponse => todo!(),
            HandshakeStateInner::InitiatorResponse => todo!(),
        }
    }

    // fn send_failure_message(
    //     &mut self,
    //     config: &PluginConfiguration,
    //     shared: &mut ConnectionShared,
    // ) {
    //     let buffer = Vec::with_capacity(8);
    //     shared.reliability.local_sequence.clone();
    //     shared.reliability.
    // }
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