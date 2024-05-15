mod postupdate;
mod preupdate;

pub(super) use postupdate::PostUpdateTickData;
pub(super) use preupdate::PreUpdateTickData;

use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::plugin::PluginConfiguration;
use super::{handshake::{HandshakeOutcome, HandshakeStateMachine}, shared::ConnectionShared, ConnectionState};

/// State machine for a connection.
pub(super) struct ConnectionStateMachine {
    inner: MachineInner,
}

impl ConnectionStateMachine {
    pub fn new(shared: &ConnectionShared) -> Self {
        Self { 
            inner: MachineInner::Handshaking(HandshakeStateMachine::new(shared.direction()))
        }
    }

    pub(super) fn state(&self) -> ConnectionState {
        match self.inner {
            MachineInner::Handshaking(_) => ConnectionState::Handshaking,
            MachineInner::Established => ConnectionState::Established,
            MachineInner::Closing => ConnectionState::Closing,
            MachineInner::Closed => ConnectionState::Closed,
        }
    }
}

enum MachineInner {
    Handshaking(HandshakeStateMachine),
    Established,
    Closing,
    Closed,
}