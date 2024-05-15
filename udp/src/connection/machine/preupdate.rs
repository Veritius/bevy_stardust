use super::*;

impl ConnectionStateMachine {
    pub fn tick_preupdate(
        &mut self,
        shared: &mut ConnectionShared,
        context: PreUpdateTickData,
    ) {
        'outer: loop {
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
                                continue 'outer;
                            },
                            Some(HandshakeOutcome::FailedHandshake) => {
                                self.inner = MachineInner::Closed;
                                continue 'outer;
                            },
                            None => {},
                        }
                    }
                },
                MachineInner::Established => todo!(),
                MachineInner::Closing => todo!(),
                MachineInner::Closed => { break 'outer },
            }
        }
    }
}