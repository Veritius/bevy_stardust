use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::plugin::PluginConfiguration;
use super::{handshake::HandshakeStateMachine, shared::ConnectionShared};

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

    pub fn tick_preupdate(
        &mut self,
        shared: &mut ConnectionShared,
        context: PreUpdateTickData,
    ) {
        todo!()
    }

    pub fn tick_postupdate(
        &mut self,
        shared: &mut ConnectionShared,
        context: PostUpdateTickData,
    ) {
        todo!()
    }
}

/// Data used by [`tick_preupdate`](ConnectionInner::tick_preupdate)
pub(super) struct PreUpdateTickData<'a> {
    pub config: &'a PluginConfiguration,
    pub registry: &'a ChannelRegistryInner,
    pub messages: Option<Mut<'a, NetworkMessages<Incoming>>>,
}

/// Data used by [`tick_postupdate`](ConnectionInner::tick_postupdate)
pub(super) struct PostUpdateTickData<'a> {
    pub config: &'a PluginConfiguration,
    pub registry: &'a ChannelRegistryInner,
    pub messages: Option<Ref<'a, NetworkMessages<Outgoing>>>,
}

enum MachineInner {
    Handshaking(HandshakeStateMachine),
    Established,
}