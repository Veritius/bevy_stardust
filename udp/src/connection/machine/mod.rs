mod postupdate;
mod preupdate;

pub(super) use postupdate::PostUpdateTickData;
pub(super) use preupdate::PreUpdateTickData;

use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::plugin::PluginConfiguration;
use super::{handshake::{HandshakeOutcome, HandshakeStateMachine}, shared::ConnectionShared};

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
}

enum MachineInner {
    Handshaking(HandshakeStateMachine),
    Established,
    Closing,
    Closed,
}