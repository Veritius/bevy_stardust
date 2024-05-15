use super::*;

impl ConnectionStateMachine {
    pub fn tick_postupdate(
        &mut self,
        shared: &mut ConnectionShared,
        context: PostUpdateTickData,
    ) {
        'outer: loop {
            match &mut self.inner {
                MachineInner::Handshaking(_) => todo!(),
                MachineInner::Established => todo!(),
                MachineInner::Closing => todo!(),
                MachineInner::Closed => { break 'outer }
            }
        }
    }
}