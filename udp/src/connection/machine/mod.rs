mod postupdate;
mod preupdate;
mod events;

pub(super) use postupdate::PostUpdateTickData;
pub(super) use preupdate::PreUpdateTickData;
pub(super) use events::ConnectionEvent;

use std::collections::VecDeque;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::plugin::PluginConfiguration;
use super::{established::EstablishedStateMachine, handshake::{HandshakeOutcome, HandshakeStateMachine}, shared::ConnectionShared, ConnectionState};

/// State machine for a connection.
pub(super) struct ConnectionStateMachine {
    inner: MachineInner,
    events: VecDeque<ConnectionEvent>,
}

impl ConnectionStateMachine {
    pub fn new(shared: &ConnectionShared) -> Self {
        Self { 
            inner: MachineInner::Handshaking(HandshakeStateMachine::new(shared.direction())),
            events: VecDeque::with_capacity(2),
        }
    }

    pub(super) fn state(&self) -> ConnectionState {
        match self.inner {
            MachineInner::Handshaking(_) => ConnectionState::Handshaking,
            MachineInner::Established(_)=> ConnectionState::Established,
            MachineInner::Closing => ConnectionState::Closing,
            MachineInner::Closed => ConnectionState::Closed,
        }
    }

    #[inline]
    pub(super) fn pop_event(&mut self) -> Option<ConnectionEvent> {
        self.events.pop_front()
    }
}

enum MachineInner {
    Handshaking(HandshakeStateMachine),
    Established(EstablishedStateMachine),
    Closing,
    Closed,
}