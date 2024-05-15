use super::*;

/// Data used by [`tick_preupdate`](ConnectionInner::tick_preupdate)
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
                            self.inner = MachineInner::Established;
                            self.tick_preupdate(shared, context);
                            return;
                        },
                        Some(HandshakeOutcome::FailedHandshake { reason }) => {
                            self.inner = MachineInner::Closed;
                            self.tick_preupdate(shared, context);
                            return;
                        },
                        None => {},
                    }
                }
            },
            MachineInner::Established => todo!(),
            MachineInner::Closing => todo!(),
            MachineInner::Closed => { return },
        }
    }
}