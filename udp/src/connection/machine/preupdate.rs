use crate::prelude::ConnectionDirection;

use super::*;

/// Data used by [`tick_preupdate`](ConnectionStateMachine::tick_preupdate)
pub(in crate::connection) struct PreUpdateTickData<'a> {
    pub config: &'a PluginConfiguration,
    pub registry: &'a ChannelRegistryInner,
    pub messages: Option<Mut<'a, NetworkMessages<Incoming>>>,
}

impl ConnectionStateMachine {
    pub fn tick_preupdate(
        &mut self,
        shared: &mut ConnectionShared,
        context: PreUpdateTickData,
    ) {
        match &mut self.inner {
            MachineInner::Handshaking(handshake) => {
                while let Some(packet) = shared.recv_queue.pop_front() {
                    let outcome = handshake.recv(
                        packet,
                        context.config,
                        shared,
                    );

                    match outcome {
                        Some(HandshakeOutcome::FinishedHandshake) => {
                            let mut swp = HandshakeStateMachine::new(ConnectionDirection::Client);
                            std::mem::swap(&mut swp, handshake);
                            self.inner = MachineInner::Established(EstablishedStateMachine::new(swp));
                            self.events.push_back(ConnectionEvent::BecameEstablished);
                            return;
                        },
                        Some(HandshakeOutcome::FailedHandshake { reason }) => {
                            self.inner = MachineInner::Closed;
                            return;
                        },
                        None => {},
                    }
                }
            },

            MachineInner::Established(machine) => {
                machine.tick_preupdate(shared, context);
            },

            MachineInner::Closing => todo!(),

            MachineInner::Closed => { return },
        }
    }
}